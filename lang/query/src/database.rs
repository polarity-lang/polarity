use crate::result::DriverError;
use crate::{cache::*, Error, FileSource};
use std::rc::Rc;
use std::sync::Arc;

use crate::dependency_graph::DependencyGraph;
use ast::Exp;
use ast::HashSet;
use elaborator::normalizer::normalize::Normalize;
use elaborator::LookupTable;
use parser::cst;
use parser::cst::decls::UseDecl;
use renaming::Rename;
use url::Url;

use crate::fs::*;
use crate::info::*;

use rust_lapper::Lapper;

/// A database tracking a set of source files
pub struct Database {
    /// The source provider of the files (file system or in-memory)
    pub source: Box<dyn FileSource>,
    /// Dependency graph for each module
    pub deps: DependencyGraph,
    /// The source code text of each file
    pub files: Cache<codespan::File<String>>,
    /// The CST of each file (once parsed)
    pub cst: Cache<Result<Arc<cst::decls::Module>, Error>>,
    /// The symbol table constructed during lowering
    pub cst_lookup_table: Cache<lowering::LookupTable>,
    /// The AST of each file (once parsed and lowered, may be type-annotated)
    pub ast: Cache<Result<Arc<ast::Module>, Error>>,
    /// The symbol table constructed during typechecking
    pub ast_lookup_table: Cache<elaborator::LookupTable>,
    /// Hover information for spans
    pub info_by_id: Cache<Lapper<u32, Info>>,
    /// Spans of top-level items
    pub item_by_id: Cache<Lapper<u32, Item>>,
}

impl Database {
    /// Create a new database that only keeps files in memory
    pub fn in_memory() -> Self {
        Self::from_source(InMemorySource::new())
    }

    /// Create a new database with the given source
    pub fn from_source(source: impl FileSource + 'static) -> Self {
        Self {
            source: Box::new(source),
            files: Cache::default(),
            deps: DependencyGraph::default(),
            cst: Cache::default(),
            cst_lookup_table: Cache::default(),
            ast: Cache::default(),
            ast_lookup_table: Cache::default(),
            info_by_id: Cache::default(),
            item_by_id: Cache::default(),
        }
    }

    /// Get the source of the files
    pub fn source(&self) -> &dyn FileSource {
        &*self.source
    }

    /// Get a mutable reference to the source of the files
    pub fn source_mut(&mut self) -> &mut Box<dyn FileSource> {
        &mut self.source
    }

    /// Invalidate the file behind the given URI and all its reverse dependencies
    pub fn invalidate(&mut self, uri: &Url) -> Result<(), Error> {
        self.build_dependency_dag()?;
        let mut rev_deps: HashSet<Url> =
            self.deps.reverse_dependencies(uri).into_iter().cloned().collect();
        log::debug!(
            "Invalidating {} and its reverse dependencies: {:?}",
            uri,
            rev_deps.iter().map(ToString::to_string).collect::<Vec<_>>()
        );
        rev_deps.insert(uri.clone());
        for rev_dep in &rev_deps {
            self.files.invalidate(rev_dep);
            self.cst.invalidate(rev_dep);
            self.cst_lookup_table.invalidate(rev_dep);
            self.ast.invalidate(rev_dep);
            self.ast_lookup_table.invalidate(rev_dep);
            self.info_by_id.invalidate(rev_dep);
            self.item_by_id.invalidate(rev_dep);
        }
        Ok(())
    }

    pub fn run(&mut self, uri: &Url) -> Result<Option<Box<Exp>>, Error> {
        let ast = self.load_module(uri)?;

        let main = ast.find_main();

        match main {
            Some(exp) => {
                let nf = exp.normalize_in_empty_env(&ast)?;
                Ok(Some(nf))
            }
            None => Ok(None),
        }
    }

    pub fn pretty_error(&self, uri: &Url, err: Error) -> miette::Report {
        let miette_error: miette::Error = err.into();
        let source = &self.files.get_even_if_stale(uri).unwrap().source;
        miette_error.with_source_code(miette::NamedSource::new(uri, source.to_owned()))
    }
    pub fn load_ast(
        &mut self,
        uri: &Url,
        cst_lookup_table: &mut lowering::LookupTable,
        ast_lookup_table: &mut LookupTable,
    ) -> Result<Arc<ast::Module>, Error> {
        log::trace!("Loading AST: {}", uri);

        match self.ast.get_unless_stale(uri) {
            Some(ast) => {
                *cst_lookup_table = self.cst_lookup_table.get_even_if_stale(uri).unwrap().clone();
                *ast_lookup_table = self.ast_lookup_table.get_even_if_stale(uri).unwrap().clone();
                ast.clone()
            }
            None => {
                log::trace!("AST is stale, reloading");
                let ust = self.load_ust(uri, cst_lookup_table);
                let ast = ust
                    .and_then(|ust| {
                        let tst = elaborator::typechecker::check_with_lookup_table(
                            Rc::new(ust),
                            ast_lookup_table,
                        )
                        .map_err(Error::Type)?;
                        Ok(tst)
                    })
                    .map(Arc::new);
                self.ast.insert(uri.clone(), ast.clone());
                self.ast_lookup_table.insert(uri.clone(), ast_lookup_table.clone());
                self.cst_lookup_table.insert(uri.clone(), cst_lookup_table.clone());
                if let Ok(module) = &ast {
                    let (info_lapper, item_lapper) = collect_info(module.clone());
                    self.info_by_id.insert(uri.clone(), info_lapper);
                    self.item_by_id.insert(uri.clone(), item_lapper);
                }
                ast
            }
        }
    }

    pub fn load_ust(
        &mut self,
        uri: &Url,
        cst_lookup_table: &mut lowering::LookupTable,
    ) -> Result<ast::Module, Error> {
        log::trace!("Loading UST: {}", uri);

        let cst = self.load_cst(uri)?;
        log::debug!("Lowering module");
        lowering::lower_module_with_lookup_table(&cst, cst_lookup_table).map_err(Error::Lowering)
    }

    pub fn load_cst(&mut self, uri: &Url) -> Result<Arc<cst::decls::Module>, Error> {
        match self.cst.get_unless_stale(uri) {
            Some(cst) => cst.clone(),
            None => {
                let source = self.load_source(uri)?;
                let module = {
                    let source: &str = &source;
                    log::debug!("Parsing module: {}", uri);
                    parser::parse_module(uri.clone(), source).map_err(Error::Parser)
                }
                .map(Arc::new);
                self.cst.insert(uri.clone(), module.clone());
                module
            }
        }
    }

    pub fn load_source(&mut self, uri: &Url) -> Result<String, Error> {
        match self.files.get_unless_stale(uri) {
            Some(file) => Ok(file.source().to_string()),
            None => {
                let source = self.source.read_to_string(uri)?;
                let file = codespan::File::new(uri.as_str().into(), source.clone());
                self.files.insert(uri.clone(), file);
                Ok(source)
            }
        }
    }

    pub fn write_source(&mut self, uri: &Url, source: &str) -> Result<(), Error> {
        self.invalidate(uri)?;
        self.source.write_string(uri, source).map_err(|err| err.into())
    }

    pub fn print_to_string(&mut self, uri: &Url) -> Result<String, Error> {
        let module =
            self.load_ast(uri, &mut lowering::LookupTable::default(), &mut LookupTable::default())?;
        let module = (*module).clone().rename();
        Ok(printer::Print::print_to_string(&module, None))
    }

    pub fn load_module(&mut self, module_uri: &Url) -> Result<Arc<ast::Module>, Error> {
        log::debug!("Loading module: {}", module_uri);
        self.build_dependency_dag()?;

        log::trace!("");
        log::trace!("Dependency graph:");
        log::trace!("");
        self.deps.print_dependency_tree();
        log::trace!("");

        let mut cst_lookup_table = lowering::LookupTable::default();
        let mut ast_lookup_table = LookupTable::default();
        self.load_module_impl(&mut cst_lookup_table, &mut ast_lookup_table, module_uri)
    }

    pub fn load_imports(
        &mut self,
        module_uri: &Url,
        cst_lookup_table: &mut lowering::LookupTable,
        ast_lookup_table: &mut LookupTable,
    ) -> Result<(), Error> {
        self.build_dependency_dag()?;
        let empty_vec = Vec::new();
        let direct_deps = self.deps.get(module_uri).unwrap_or(&empty_vec).clone();
        for direct_dep in direct_deps {
            self.load_module_impl(cst_lookup_table, ast_lookup_table, &direct_dep)?;
        }
        Ok(())
    }

    /// Builds the dependency DAG for a given module and checks for cycles.
    ///
    /// Returns a `HashMap` where each key is a module `Url` and the corresponding value
    /// is a vector of `Url`s representing the modules it depends on.
    ///
    /// # Errors
    ///
    /// Returns an error if a cycle is detected or if a module cannot be found or loaded.
    pub fn build_dependency_dag(&mut self) -> Result<(), Error> {
        let mut visited = HashSet::default();
        let mut stack = Vec::new();
        let mut graph = DependencyGraph::default();
        let modules: Vec<Url> = self.files.keys().cloned().collect();
        for module_uri in modules {
            self.visit_module(&module_uri, &mut visited, &mut stack, &mut graph)?;
        }
        self.deps = graph;
        Ok(())
    }

    /// Recursively visits a module, adds its dependencies to the graph, and checks for cycles.
    fn visit_module(
        &mut self,
        module_uri: &Url,
        visited: &mut HashSet<Url>,
        stack: &mut Vec<Url>,
        graph: &mut DependencyGraph,
    ) -> Result<(), Error> {
        if stack.contains(module_uri) {
            // Cycle detected
            let cycle = stack.to_vec();
            return Err(DriverError::ImportCycle(module_uri.clone(), cycle).into());
        }

        if visited.contains(module_uri) {
            // Module already processed
            return Ok(());
        }

        visited.insert(module_uri.clone());
        stack.push(module_uri.clone());

        let module = self.load_cst(module_uri)?;

        // Collect dependencies from `use` declarations
        let mut dependencies = Vec::new();
        for use_decl in &module.use_decls {
            let UseDecl { path, .. } = use_decl;
            // Resolve the module name to a `Url`
            let dep_url = self.resolve_module_name(path, module_uri)?;
            dependencies.push(dep_url.clone());

            // Recursively visit the dependency
            self.visit_module(&dep_url, visited, stack, graph)?;
        }

        // Add the module and its dependencies to the graph
        graph.insert(module_uri.clone(), dependencies);

        stack.pop();
        Ok(())
    }

    /// Resolves a module name to a `Url` relative to the current module.
    fn resolve_module_name(&self, name: &str, current_module: &Url) -> Result<Url, Error> {
        current_module.join(name).map_err(|err| DriverError::Url(err).into())
    }

    fn load_module_impl(
        &mut self,
        cst_lookup_table: &mut lowering::LookupTable,
        ast_lookup_table: &mut LookupTable,
        module_uri: &Url,
    ) -> Result<Arc<ast::Module>, Error> {
        let empty_vec = Vec::new();
        let direct_dependencies = self.deps.get(module_uri).unwrap_or(&empty_vec).clone();

        for dep_url in direct_dependencies {
            self.load_module_impl(cst_lookup_table, ast_lookup_table, &dep_url)?;
        }

        self.load_ast(module_uri, cst_lookup_table, ast_lookup_table)
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod path_support {
    use super::*;

    impl Database {
        /// Create a new database tracking the folder at the given path
        /// If the path is a file, the parent directory is tracked
        pub fn from_path<P: AsRef<std::path::Path>>(path: P) -> Self {
            let path = path.as_ref();
            let path = if path.is_dir() {
                path
            } else {
                path.parent().expect("Could not get parent directory")
            };
            Self::from_source(FileSystemSource::new(path))
        }

        /// Create a new database tracking the current working directory
        pub fn from_cwd() -> Self {
            Self::from_path(std::env::current_dir().expect("Could not get current directory"))
        }

        /// Open a file by its path and load it into the database
        pub fn resolve_path<P: AsRef<std::path::Path>>(&mut self, path: P) -> Result<Url, Error> {
            let path = path.as_ref().canonicalize().expect("Could not canonicalize path");
            Ok(Url::from_file_path(path).expect("Could not convert path to URI"))
        }
    }
}
