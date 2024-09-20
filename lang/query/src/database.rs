use crate::{cache::*, Error, FileSource};
use std::rc::Rc;
use std::sync::Arc;

use crate::dependency_graph::DependencyGraph;
use ast::Exp;
use ast::HashSet;
use elaborator::normalizer::normalize::Normalize;
use elaborator::LookupTable;
use parser::cst;
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
