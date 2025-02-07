use url::Url;

use ast::{HashMap, HashSet};

#[derive(Default)]
pub struct DependencyGraph {
    graph: HashMap<Url, Vec<Url>>,
}

impl DependencyGraph {
    pub fn get(&self, url: &Url) -> Option<&Vec<Url>> {
        self.graph.get(url)
    }

    pub fn invalidate(&mut self, url: &Url) {
        self.graph.remove(url);
    }

    pub fn insert(&mut self, url: Url, deps: Vec<Url>) {
        self.graph.insert(url, deps);
    }

    pub fn reverse_dependencies<'a>(&'a self, uri: &'a Url) -> Vec<&'a Url> {
        if !self.graph.contains_key(uri) {
            return Vec::new();
        }
        let mut closure = Vec::new();
        let mut stack = vec![uri];
        let mut visited = HashSet::default();
        while let Some(url) = stack.pop() {
            if visited.insert(url.clone()) {
                closure.push(url);
                let rev_deps = self
                    .graph
                    .iter()
                    .filter_map(|(rev_dep, v)| if v.contains(url) { Some(rev_dep) } else { None })
                    .collect::<Vec<_>>();
                stack.extend(rev_deps);
            }
        }
        closure
    }

    /// Prints the dependency graph as an indented tree.
    ///
    /// Each module is printed with its dependencies indented below it.
    ///
    /// # Arguments
    ///
    /// * `graph` - A reference to the dependency graph represented as a `DependencyGraph`.
    pub fn print_dependency_tree(&self) {
        let mut visited = HashSet::default();
        for module_uri in self.graph.keys() {
            self.print_module_dependencies(module_uri, &mut visited, 0);
        }
    }

    /// Helper function to recursively print module dependencies with indentation.
    ///
    /// # Arguments
    ///
    /// * `module_uri` - The URL of the current module being printed.
    /// * `graph` - A reference to the dependency graph.
    /// * `visited` - A mutable reference to a `HashSet` tracking visited modules to detect cycles.
    /// * `depth` - The current depth in the dependency tree, used for indentation.
    fn print_module_dependencies(
        &self,
        module_uri: &Url,
        visited: &mut HashSet<Url>,
        depth: usize,
    ) {
        // Indentation based on the depth in the tree
        let indent = "  ".repeat(depth);

        // Check for cycles
        if !visited.insert(module_uri.clone()) {
            log::trace!("{}{} (already visited)", indent, url_to_label(module_uri));
            return;
        }

        log::trace!("{}{}", indent, url_to_label(module_uri));

        if let Some(dependencies) = self.get(module_uri) {
            for dep_url in dependencies {
                self.print_module_dependencies(dep_url, visited, depth + 1);
            }
        }

        // Remove the module from the visited set when unwinding the recursion
        visited.remove(module_uri);
    }
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
