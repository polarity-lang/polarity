use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use elaborator::typechecker::lookup_table::LookupTable;
use parser::cst::decls::UseDecl;
use url::Url;

use crate::DatabaseViewMut;
use crate::{result::DriverError, Database, Error};

impl DatabaseViewMut<'_> {
    pub fn load_module(&mut self) -> Result<Arc<ast::Module>, Error> {
        self.database.load_module(&self.url)
    }
}

impl Database {
    fn load_module(&mut self, module_url: &Url) -> Result<Arc<ast::Module>, Error> {
        log::debug!("Loading module: {}", module_url);
        let deps = self.build_dependency_dag(module_url)?;

        log::trace!("");
        log::trace!("Dependency graph:");
        log::trace!("");
        print_dependency_tree(&deps);
        log::trace!("");

        fn load_module_impl(
            db: &mut Database,
            deps: &HashMap<Url, Vec<Url>>,
            cst_lookup_table: &mut lowering::LookupTable,
            ast_lookup_table: &mut LookupTable,
            module_url: &Url,
        ) -> Result<Arc<ast::Module>, Error> {
            let empty_vec = Vec::new();
            let direct_dependencies = deps.get(module_url).unwrap_or(&empty_vec);

            for dep_url in direct_dependencies {
                load_module_impl(db, deps, cst_lookup_table, ast_lookup_table, dep_url)?;
            }

            db.open_uri(module_url)?.load_ast(cst_lookup_table, ast_lookup_table)
        }

        let mut cst_lookup_table = lowering::LookupTable::default();
        let mut ast_lookup_table = LookupTable::default();
        load_module_impl(self, &deps, &mut cst_lookup_table, &mut ast_lookup_table, module_url)
    }

    /// Builds the dependency DAG for a given module and checks for cycles.
    ///
    /// Returns a `HashMap` where each key is a module `Url` and the corresponding value
    /// is a vector of `Url`s representing the modules it depends on.
    ///
    /// # Errors
    ///
    /// Returns an error if a cycle is detected or if a module cannot be found or loaded.
    fn build_dependency_dag(&mut self, module_url: &Url) -> Result<HashMap<Url, Vec<Url>>, Error> {
        let mut visited = HashSet::new();
        let mut stack = Vec::new();
        let mut graph = HashMap::new();
        self.visit_module(module_url, &mut visited, &mut stack, &mut graph)?;
        Ok(graph)
    }

    /// Recursively visits a module, adds its dependencies to the graph, and checks for cycles.
    fn visit_module(
        &mut self,
        module_url: &Url,
        visited: &mut HashSet<Url>,
        stack: &mut Vec<Url>,
        graph: &mut HashMap<Url, Vec<Url>>,
    ) -> Result<(), Error> {
        if stack.contains(module_url) {
            // Cycle detected
            let cycle = stack.to_vec();
            return Err(DriverError::ImportCycle(module_url.clone(), cycle).into());
        }

        if visited.contains(module_url) {
            // Module already processed
            return Ok(());
        }

        visited.insert(module_url.clone());
        stack.push(module_url.clone());

        let module = self.open_uri(module_url)?.load_cst()?;

        // Collect dependencies from `use` declarations
        let mut dependencies = Vec::new();
        for use_decl in &module.use_decls {
            let UseDecl { path, .. } = use_decl;
            // Resolve the module name to a `Url`
            let dep_url = self.resolve_module_name(path, module_url)?;
            dependencies.push(dep_url.clone());

            // Recursively visit the dependency
            self.visit_module(&dep_url, visited, stack, graph)?;
        }

        // Add the module and its dependencies to the graph
        graph.insert(module_url.clone(), dependencies);

        stack.pop();
        Ok(())
    }

    /// Resolves a module name to a `Url` relative to the current module.
    fn resolve_module_name(&self, name: &str, current_module: &Url) -> Result<Url, Error> {
        current_module.join(name).map_err(|err| DriverError::Url(err).into())
    }
}

/// Prints the dependency graph as an indented tree.
///
/// Each module is printed with its dependencies indented below it.
/// This function handles cycles by keeping track of visited modules.
///
/// # Arguments
///
/// * `graph` - A reference to the dependency graph represented as a `HashMap<Url, Vec<Url>>`.
pub fn print_dependency_tree(graph: &HashMap<Url, Vec<Url>>) {
    let mut visited = HashSet::new();
    for module_url in graph.keys() {
        print_module_dependencies(module_url, graph, &mut visited, 0);
    }
}

/// Helper function to recursively print module dependencies with indentation.
///
/// # Arguments
///
/// * `module_url` - The URL of the current module being printed.
/// * `graph` - A reference to the dependency graph.
/// * `visited` - A mutable reference to a `HashSet` tracking visited modules to detect cycles.
/// * `depth` - The current depth in the dependency tree, used for indentation.
fn print_module_dependencies(
    module_url: &Url,
    graph: &HashMap<Url, Vec<Url>>,
    visited: &mut HashSet<Url>,
    depth: usize,
) {
    // Indentation based on the depth in the tree
    let indent = "  ".repeat(depth);

    // Check for cycles
    if !visited.insert(module_url.clone()) {
        log::trace!("{}{} (already visited)", indent, url_to_label(module_url));
        return;
    }

    log::trace!("{}{}", indent, url_to_label(module_url));

    if let Some(dependencies) = graph.get(module_url) {
        for dep_url in dependencies {
            print_module_dependencies(dep_url, graph, visited, depth + 1);
        }
    }

    // Remove the module from the visited set when unwinding the recursion
    visited.remove(module_url);
}

/// Helper function to convert a `Url` to a label suitable for display.
///
/// This function extracts the file name from the URL's path for concise display.
///
/// # Arguments
///
/// * `url` - The URL to convert.
///
/// # Returns
///
/// A `String` representing the file name or the full path if extraction fails.
fn url_to_label(url: &Url) -> String {
    // Extract the file name from the path
    if let Some(path_segments) = url.path_segments() {
        if let Some(file_name) = path_segments.last() {
            return file_name.to_string();
        }
    }
    url.path().to_string()
}
