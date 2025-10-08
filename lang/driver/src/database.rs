use std::rc::Rc;
use std::sync::Arc;

use lsp_types::HoverContents;
use polarity_lang_ast::rename::Rename;
use polarity_lang_miette_util::codespan::Span;
use url::Url;

use polarity_lang_ast::Exp;
use polarity_lang_ast::HashSet;
use polarity_lang_ast::Zonk;
use polarity_lang_backend::ast2ir::traits::ToIR;
use polarity_lang_backend::ir;
use polarity_lang_elaborator::normalizer::normalize::Normalize;
use polarity_lang_elaborator::{ModuleTypeInfoTable, TypeInfoTable, build_type_info_table};
use polarity_lang_lowering::{ModuleSymbolTable, SymbolTable};
use polarity_lang_parser::cst;
use polarity_lang_parser::cst::decls::UseDecl;

use crate::codespan::File;
use crate::dependency_graph::DependencyGraph;
use crate::fs::*;
use crate::info::*;
use crate::result::DriverError;
use crate::{Error, FileSource, cache::*};

use rust_lapper::Lapper;

/// A database tracking a set of source files
pub struct Database {
    /// The source provider of the files (file system or in-memory)
    pub source: Box<dyn FileSource>,
    /// Dependency graph for each module
    pub deps: DependencyGraph,
    /// The source code text of each file
    pub files: Cache<crate::codespan::File>,
    /// The CST of each file (once parsed)
    pub cst: Cache<Result<Arc<cst::decls::Module>, Error>>,
    /// The symbol table constructed during lowering
    pub symbol_table: Cache<Arc<polarity_lang_lowering::ModuleSymbolTable>>,
    /// The lowered, but not yet typechecked, UST
    pub ust: Cache<Result<Arc<polarity_lang_ast::Module>, Error>>,
    /// The typechecked AST of a module
    pub ast: Cache<Result<Arc<polarity_lang_ast::Module>, Error>>,
    /// The IR of a module
    pub ir: Cache<Result<Arc<ir::Module>, Error>>,
    /// The type info table, either open or closed
    pub type_info_table: Cache<OpenClosed>,
    /// Hover information for spans
    pub hover_by_id: Cache<Lapper<u32, HoverContents>>,
    /// Goto information for spans
    pub goto_by_id: Cache<Lapper<u32, (Url, Span)>>,
    /// Spans of top-level items
    pub item_by_id: Cache<Lapper<u32, Item>>,
}

/// Open or closed type info table
pub enum OpenClosed {
    /// The type info table constructed *before* elaboration.
    /// This table is used to lookup top-level signatures in the module that is currently being checked.
    /// "open" means there may be unsolved metavariables in the table.
    Open(polarity_lang_elaborator::ModuleTypeInfoTable),
    /// The type info table constructed *after* elaboration.
    /// This table is used to lookup top-level signatures in dependencies.
    /// "closed" means that no unsolved metavariables occur in the table.
    Closed(polarity_lang_elaborator::ModuleTypeInfoTable),
}

impl Database {
    // Core API
    //
    // The core API of the Database consists of functions which have the following forms:
    //
    // ```text
    // pub fn xxx(&mut self, uri: &Url) -> Result<xxx, Error>
    // fn recompute_xxx(&mut self, uri: &Url) -> Result<(xxx, Error>
    // ```
    // where `xxx` can be the cst, ust, ast, or any other sort of information about a module.
    // These functions are all implemented in a similar way.
    //
    // The function `xxx(&mut self, uri: &Url)` checks whether the desired object is already in the
    // cache. If it is in the cache and isn't marked as stale we immediately return the object.
    // Otherwise we call `recompute_xxx` which contains the logic to compute the object anew
    // and put it back in the cache.
    //
    // The function `recompute_xxx(&mut self, uri: &Url)` generally proceeds in the following way:
    //
    // 1. We look into the dependency graph to find out what the direct dependencies
    //    of the module are.
    // 2. For each of the direct dependencies we use the `xxx(...)` functions to obtain the
    //    information that is required to recompute the object. For example, we obtain the
    //    symbol tables for renaming or the lookup tables for typechecking a module.
    //    These calls can trigger further computations if the information is not in one of the
    //    caches.

    // Core API: Source
    //
    //

    pub async fn source(&mut self, uri: &Url) -> Result<String, Error> {
        match self.files.get_unless_stale(uri) {
            Some(file) => {
                log::debug!("Found source in cache: {uri}");
                Ok(file.source.to_string())
            }
            None => self.recompute_source(uri).await,
        }
    }

    async fn recompute_source(&mut self, uri: &Url) -> Result<String, Error> {
        log::debug!("Recomputing source for: {uri}");
        let source = self.source.read_to_string(uri).await?;
        let file = crate::codespan::File::new(source.clone());
        self.files.insert(uri.clone(), file);
        Ok(source)
    }

    // Core API: CST (Concrete Syntax Tree)
    //
    //

    pub async fn cst(&mut self, uri: &Url) -> Result<Arc<cst::decls::Module>, Error> {
        match self.cst.get_unless_stale(uri) {
            Some(cst) => {
                log::debug!("Found cst in cache: {uri}");
                cst.clone()
            }
            None => self.recompute_cst(uri).await,
        }
    }

    async fn recompute_cst(&mut self, uri: &Url) -> Result<Arc<cst::decls::Module>, Error> {
        log::debug!("Recomputing cst for: {uri}");
        let source = self.source(uri).await?;
        let module = polarity_lang_parser::parse_module(uri.clone(), &source)
            .map_err(Error::Parser)
            .map(Arc::new);
        self.cst.insert(uri.clone(), module.clone());
        module
    }

    // Core API: SymbolTable
    //
    //

    pub async fn symbol_table(&mut self, uri: &Url) -> Result<Arc<ModuleSymbolTable>, Error> {
        match self.symbol_table.get_unless_stale(uri) {
            Some(symbol_table) => {
                log::debug!("Found symbol table in cache: {uri}");
                Ok(symbol_table.clone())
            }
            None => self.recompute_symbol_table(uri).await,
        }
    }

    async fn recompute_symbol_table(&mut self, uri: &Url) -> Result<Arc<ModuleSymbolTable>, Error> {
        log::debug!("Recomputing symbol table for: {uri}");
        let cst = self.cst(uri).await?;
        let module_symbol_table = polarity_lang_lowering::build_symbol_table(&cst).map(Arc::new)?;
        self.symbol_table.insert(uri.clone(), module_symbol_table.clone());
        Ok(module_symbol_table)
    }

    // Core API: UST
    //
    //

    pub async fn ust(&mut self, uri: &Url) -> Result<Arc<polarity_lang_ast::Module>, Error> {
        match self.ust.get_unless_stale(uri) {
            Some(ust) => {
                log::debug!("Found ust in cache: {uri}");
                ust.clone()
            }
            None => self.recompute_ust(uri).await,
        }
    }

    pub async fn recompute_ust(
        &mut self,
        uri: &Url,
    ) -> Result<Arc<polarity_lang_ast::Module>, Error> {
        log::debug!("Recomputing ust for: {uri}");
        let cst = self.cst(uri).await?;
        let deps = self.deps(uri).await?;

        // Compute the SymbolTable consisting of all the
        // ModuleSymbolTables of all direct dependencies
        // and the SymbolTable from the module itself.
        let mut symbol_table = SymbolTable::default();
        let module_symbol_table = self.symbol_table(uri).await?;
        symbol_table.insert(uri.clone(), module_symbol_table);
        for dep in deps {
            let module_symbol_table = self.symbol_table(&dep).await?;
            symbol_table.insert(dep.clone(), module_symbol_table);
        }

        let ust = polarity_lang_lowering::lower_module_with_symbol_table(&cst, &symbol_table)
            .map_err(Error::Lowering)
            .map(Arc::new);

        self.ust.insert(uri.clone(), ust.clone());
        ust
    }

    // Core API: TypeInfoTable
    //
    //

    pub async fn type_info_table(&mut self, uri: &Url) -> Result<TypeInfoTable, Error> {
        Box::pin(async move {
            let deps = self.deps(uri).await?;

            // Compute the type info table
            let mut info_table = TypeInfoTable::default();
            let mod_info_table = self.module_type_info_table(uri, false).await?;
            info_table.insert(uri.clone(), mod_info_table);
            for dep_url in deps {
                let mod_info_table = self.module_type_info_table(&dep_url, true).await?;
                info_table.insert(dep_url.clone(), mod_info_table);
            }

            Ok(info_table)
        })
        .await
    }

    /// Get the type the type info table for a module with the given `uri`
    ///
    /// # Parameters
    ///
    /// - `uri`:    The module URI for which we want the type info table
    /// - `closed`: Whether to return the open or the closed type info table.
    ///   The open type info table is used to lookup top-level signatures while elaborating the very same module.
    ///   It may contain unsolved metavariables.
    ///   The closed type info table is used to lookup top-level signatures of module dependencies.
    ///   It contains no unsolved metavariables.
    ///
    /// # Returns
    ///
    /// The type info table for the requested module.
    ///
    /// # Ensures
    ///
    /// - If `closed = true`, no unsolved metavariables occur in the type info table.
    pub async fn module_type_info_table(
        &mut self,
        uri: &Url,
        closed: bool,
    ) -> Result<ModuleTypeInfoTable, Error> {
        match self.type_info_table.get_unless_stale(uri) {
            Some(open_closed) => {
                let table = match (open_closed, closed) {
                    (OpenClosed::Open(_), true) => {
                        // Closed table is requested, but only open table is available
                        return self.recompute_type_info_table(uri, closed).await;
                    }
                    (OpenClosed::Open(table), false) => table,
                    (OpenClosed::Closed(table), _) => table,
                };
                log::debug!("Found type info table in cache: {uri}");
                Ok(table.clone())
            }
            None => self.recompute_type_info_table(uri, closed).await,
        }
    }

    /// Recompute type the type info table for a module with the given `uri`
    ///
    /// # Parameters
    ///
    /// - `uri`:    The module URI for which we want to recompute the type info table
    /// - `closed`: Whether to recompute the open or the closed type info table.
    ///   Computing the closed type info table requires elaboration of the module
    ///   and replacing all metavariables with their solution ("zonking").
    ///
    /// # Returns
    ///
    /// The type info table for the requested module.
    ///
    /// # Ensures
    ///
    /// - If `closed = true`, no unsolved metavariables occur in the type info table.
    pub async fn recompute_type_info_table(
        &mut self,
        uri: &Url,
        closed: bool,
    ) -> Result<ModuleTypeInfoTable, Error> {
        log::debug!(
            "Recomputing {} type info table for: {}",
            if closed { "closed" } else { "open" },
            uri
        );
        let ust = self.ust(uri).await?;
        let mut info_table = build_type_info_table(&ust);
        self.type_info_table.insert(uri.clone(), OpenClosed::Open(info_table.clone()));
        if closed {
            let ast = self.ast(uri).await?;
            info_table
                .zonk(&ast.meta_vars)
                .map_err(|err| DriverError::Impossible(err.to_string()))?;
            self.type_info_table.insert(uri.clone(), OpenClosed::Closed(info_table.clone()));
        }
        Ok(info_table)
    }

    // Core API: AST
    //
    //

    pub async fn ast(&mut self, uri: &Url) -> Result<Arc<polarity_lang_ast::Module>, Error> {
        match self.ast.get_unless_stale(uri) {
            Some(ast) => {
                log::debug!("Found ast in cache: {uri}");
                ast.clone()
            }
            None => self.recompute_ast(uri).await,
        }
    }

    pub async fn recompute_ast(
        &mut self,
        uri: &Url,
    ) -> Result<Arc<polarity_lang_ast::Module>, Error> {
        log::debug!("Recomputing ast for: {uri}");

        // Compute the type info table
        let info_table = self.type_info_table(uri).await?;

        // Typecheck module
        let ust = self.ust(uri).await.map(|x| (*x).clone())?;
        let ast = polarity_lang_elaborator::typechecker::check_with_lookup_table(
            Rc::new(ust),
            &info_table,
        )
        .map(Arc::new)
        .map_err(Error::Type);
        self.ast.insert(uri.clone(), ast.clone());
        ast
    }

    // Core API: ir
    //
    //

    pub async fn ir(&mut self, uri: &Url) -> Result<Arc<ir::Module>, Error> {
        match self.ir.get_unless_stale(uri) {
            Some(module) => {
                log::debug!("Found ir in cache: {uri}");
                module.clone()
            }
            None => self.recompute_ir(uri).await,
        }
    }

    pub async fn recompute_ir(&mut self, uri: &Url) -> Result<Arc<ir::Module>, Error> {
        log::debug!("Recomputing ir for: {uri}");

        let module = self.ast(uri).await?;

        // Convert to intermediate representation (IR)
        let ir = module.to_ir().map(Arc::new).map_err(Error::Backend);

        self.ir.insert(uri.clone(), ir.clone());

        ir
    }

    // Core API: goto_by_id
    //
    //

    pub async fn goto_by_id(&mut self, uri: &Url) -> Result<Lapper<u32, (Url, Span)>, Error> {
        match self.goto_by_id.get_unless_stale(uri) {
            Some(loc) => {
                log::debug!("Found goto_by_id in cache: {uri}");
                Ok(loc.clone())
            }
            None => self.recompute_goto_by_id(uri).await,
        }
    }

    async fn recompute_goto_by_id(&mut self, uri: &Url) -> Result<Lapper<u32, (Url, Span)>, Error> {
        log::debug!("Recomputing goto_by_id for: {uri}");
        let (_hover_lapper, location_lapper, _item_lapper) = collect_info(self, uri).await?;
        self.goto_by_id.insert(uri.clone(), location_lapper.clone());
        Ok(location_lapper)
    }

    // Core API: hover_by_id
    //
    //

    pub async fn hover_by_id(&mut self, uri: &Url) -> Result<Lapper<u32, HoverContents>, Error> {
        match self.hover_by_id.get_unless_stale(uri) {
            Some(hover) => {
                log::debug!("Found hover_by_id in cache: {uri}");
                Ok(hover.clone())
            }
            None => self.recompute_hover_by_id(uri).await,
        }
    }

    async fn recompute_hover_by_id(
        &mut self,
        uri: &Url,
    ) -> Result<Lapper<u32, HoverContents>, Error> {
        log::debug!("Recomputing hover_by_id for: {uri}");
        let (hover_lapper, _location_lapper, _item_lapper) = collect_info(self, uri).await?;
        self.hover_by_id.insert(uri.clone(), hover_lapper.clone());
        Ok(hover_lapper)
    }

    // Core API: item_by_id
    //
    //

    pub async fn item_by_id(&mut self, uri: &Url) -> Result<Lapper<u32, Item>, Error> {
        match self.item_by_id.get_unless_stale(uri) {
            Some(items) => {
                log::debug!("Found item_by_id in cache: {uri}");
                Ok(items.clone())
            }
            None => self.recompute_item_by_id(uri).await,
        }
    }

    async fn recompute_item_by_id(&mut self, uri: &Url) -> Result<Lapper<u32, Item>, Error> {
        log::debug!("Recomputing item_by_id for: {uri}");
        let (_info_lapper, _location_lapper, item_lapper) = collect_info(self, uri).await?;
        self.item_by_id.insert(uri.clone(), item_lapper.clone());
        Ok(item_lapper)
    }

    // Core API: Dependencies
    //
    //

    pub async fn deps(&mut self, uri: &Url) -> Result<Vec<Url>, Error> {
        match self.deps.get(uri) {
            Some(deps) => {
                log::debug!("Found dependencies in cache: {uri}");
                Ok(deps.clone())
            }
            None => Box::pin(self.recompute_deps(uri)).await,
        }
    }

    pub async fn recompute_deps(&mut self, uri: &Url) -> Result<Vec<Url>, Error> {
        log::debug!("Recomputing dependencies for: {uri}");
        self.source(uri).await?;
        self.build_dependency_dag().await?;
        self.deps(uri).await
    }

    // Creation
    //
    // The following methods provide various means to construct a driver instance.

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
            symbol_table: Cache::default(),
            ust: Cache::default(),
            ast: Cache::default(),
            ir: Cache::default(),
            type_info_table: Cache::default(),
            hover_by_id: Cache::default(),
            goto_by_id: Cache::default(),
            item_by_id: Cache::default(),
        }
    }

    // Utility Functions
    //
    // The following utility functions do not belong to the core API described above.

    /// Get the source of the files
    pub fn file_source(&self) -> &dyn FileSource {
        &*self.source
    }

    /// Get a mutable reference to the source of the files
    pub fn file_source_mut(&mut self) -> &mut Box<dyn FileSource> {
        &mut self.source
    }

    /// Invalidate the file behind the given URI and all its reverse dependencies
    pub async fn invalidate(&mut self, uri: &Url) {
        self.invalidate_impl(uri);
        let rev_deps: HashSet<Url> =
            self.deps.reverse_dependencies(uri).into_iter().cloned().collect();
        log::debug!(
            "Invalidating {} and its reverse dependencies: {:?}",
            uri,
            rev_deps.iter().map(ToString::to_string).collect::<Vec<_>>()
        );
        for rev_dep in &rev_deps {
            self.invalidate_impl(rev_dep);
        }
    }

    fn invalidate_impl(&mut self, uri: &Url) {
        self.files.invalidate(uri);
        self.deps.invalidate(uri);
        self.cst.invalidate(uri);
        self.symbol_table.invalidate(uri);
        self.ust.invalidate(uri);
        self.ast.invalidate(uri);
        self.type_info_table.invalidate(uri);
        self.hover_by_id.invalidate(uri);
        self.goto_by_id.invalidate(uri);
        self.item_by_id.invalidate(uri);
    }

    pub async fn run(&mut self, uri: &Url) -> Result<Option<Box<Exp>>, Error> {
        let ast = self.ast(uri).await?;

        let main = ast.find_main();
        let info_table = self.type_info_table(uri).await?;

        match main {
            Some(exp) => {
                let nf = exp.normalize_in_empty_env(&Rc::new(info_table));
                nf.map(Some).map_err(Error::Type)
            }
            None => Ok(None),
        }
    }

    pub fn pretty_error(&self, uri: &Url, err: Error) -> miette::Report {
        let miette_error: miette::Error = err.into();
        if let Some(File { source, .. }) = self.files.get_even_if_stale(uri) {
            miette_error.with_source_code(miette::NamedSource::new(uri, source.to_owned()))
        } else {
            miette_error
        }
    }

    pub async fn write_source(&mut self, uri: &Url, source: &str) -> Result<(), Error> {
        self.invalidate(uri).await;
        self.source.write_string(uri, source).await.map_err(|err| err.into())
    }

    pub async fn print_to_string(&mut self, uri: &Url) -> Result<String, Error> {
        let module = self.ust(uri).await?;
        let mut module = (*module).clone();
        module.rename();
        Ok(polarity_lang_printer::Print::print_to_string(&module, None))
    }

    pub async fn load_imports(&mut self, module_uri: &Url) -> Result<(), Error> {
        self.build_dependency_dag().await?;
        let empty_vec = Vec::new();
        let direct_deps = self.deps.get(module_uri).unwrap_or(&empty_vec).clone();
        for direct_dep in direct_deps {
            self.ast(&direct_dep).await?;
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
    pub async fn build_dependency_dag(&mut self) -> Result<(), Error> {
        let mut visited = HashSet::default();
        let mut stack = Vec::new();
        let mut graph = DependencyGraph::default();
        let modules: Vec<Url> = self.files.keys().cloned().collect();
        for module_uri in modules {
            self.visit_module(&module_uri, &mut visited, &mut stack, &mut graph).await?;
        }
        self.deps = graph;
        Ok(())
    }

    /// Recursively visits a module, adds its dependencies to the graph, and checks for cycles.
    async fn visit_module(
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

        let module = self.cst(module_uri).await?;

        // Collect dependencies from `use` declarations
        let mut dependencies = Vec::new();
        for use_decl in &module.use_decls {
            let UseDecl { path, .. } = use_decl;
            // Resolve the module name to a `Url`
            let dep_url = self.resolve_module_name(path, module_uri)?;
            dependencies.push(dep_url.clone());

            // Recursively visit the dependency
            Box::pin(self.visit_module(&dep_url, visited, stack, graph)).await?;
        }

        // Add the module and its dependencies to the graph
        graph.insert(module_uri.clone(), dependencies);

        stack.pop();
        Ok(())
    }

    /// Resolves a module name to a `Url` relative to the current module.
    pub fn resolve_module_name(&self, name: &str, current_module: &Url) -> Result<Url, Error> {
        current_module.join(name).map_err(|err| DriverError::Url(err).into())
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
