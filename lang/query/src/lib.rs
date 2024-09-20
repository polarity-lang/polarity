use std::sync::Arc;

use ast::HashSet;
use dependency_graph::DependencyGraph;
use parser::cst;
use url::Url;

use rust_lapper::Lapper;

pub use result::Error;

mod asserts;
mod cache;
mod dependency_graph;
mod edit;
mod fs;
mod info;
mod lift;
mod load;
mod modules;
mod result;
mod rt;
mod spans;
mod xfunc;

use cache::*;

pub use edit::*;
pub use fs::*;
pub use info::*;
pub use xfunc::*;

/// A database tracking a set of source files
pub struct Database {
    /// The source provider of the files (file system or in-memory)
    source: Box<dyn FileSource>,
    /// Dependency graph for each module
    deps: DependencyGraph,
    /// The source code text of each file
    files: Cache<codespan::File<String>>,
    /// The CST of each file (once parsed)
    cst: Cache<Result<Arc<cst::decls::Module>, Error>>,
    /// The symbol table constructed during lowering
    cst_lookup_table: Cache<lowering::LookupTable>,
    /// The AST of each file (once parsed and lowered, may be type-annotated)
    ast: Cache<Result<Arc<ast::Module>, Error>>,
    /// The symbol table constructed during typechecking
    ast_lookup_table: Cache<elaborator::LookupTable>,
    /// Hover information for spans
    info_by_id: Cache<Lapper<u32, Info>>,
    /// Spans of top-level items
    item_by_id: Cache<Lapper<u32, Item>>,
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
