use std::sync::Arc;

use ast::HashSet;
use elaborator::LookupTable;
use parser::cst::decls::UseDecl;
use url::Url;

use crate::{dependency_graph::DependencyGraph, result::DriverError, Database, Error};

impl Database {
    pub fn load_module(&mut self, module_uri: &Url) -> Result<Arc<ast::Module>, Error> {
        log::debug!("Loading module: {}", module_uri);
        let deps = self.load_dependency_dag(module_uri)?;

        log::trace!("");
        log::trace!("Dependency graph:");
        log::trace!("");
        deps.print_dependency_tree();
        log::trace!("");

        let mut cst_lookup_table = lowering::LookupTable::default();
        let mut ast_lookup_table = LookupTable::default();
        load_module_impl(self, &deps, &mut cst_lookup_table, &mut ast_lookup_table, module_uri)
    }

    pub fn load_imports(
        &mut self,
        module_uri: &Url,
        cst_lookup_table: &mut lowering::LookupTable,
        ast_lookup_table: &mut LookupTable,
    ) -> Result<(), Error> {
        let deps = self.load_dependency_dag(module_uri)?;
        let empty_vec = Vec::new();
        let direct_deps = deps.get(module_uri).unwrap_or(&empty_vec);
        for direct_dep in direct_deps {
            load_module_impl(self, &deps, cst_lookup_table, ast_lookup_table, direct_dep)?;
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
    pub fn load_dependency_dag(&mut self, module_uri: &Url) -> Result<Arc<DependencyGraph>, Error> {
        if let Some(deps) = self.deps.get_unless_stale(module_uri) {
            return Ok(deps.clone());
        }
        let mut visited = HashSet::default();
        let mut stack = Vec::new();
        let mut graph = DependencyGraph::default();
        self.visit_module(module_uri, &mut visited, &mut stack, &mut graph)?;

        self.deps.insert(module_uri.clone(), Arc::new(graph));
        Ok(self.deps.get_even_if_stale(module_uri).unwrap().clone())
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
}

fn load_module_impl(
    db: &mut Database,
    deps: &DependencyGraph,
    cst_lookup_table: &mut lowering::LookupTable,
    ast_lookup_table: &mut LookupTable,
    module_uri: &Url,
) -> Result<Arc<ast::Module>, Error> {
    let empty_vec = Vec::new();
    let direct_dependencies = deps.get(module_uri).unwrap_or(&empty_vec);

    for dep_url in direct_dependencies {
        load_module_impl(db, deps, cst_lookup_table, ast_lookup_table, dep_url)?;
    }

    db.load_ast(module_uri, cst_lookup_table, ast_lookup_table)
}
