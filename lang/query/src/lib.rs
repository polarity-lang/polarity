use std::sync::Arc;

use url::Url;

use ast::HashMap;
use rust_lapper::Lapper;

pub use result::Error;

mod asserts;
mod fs;
mod info;
mod result;
mod view;
mod view_mut;

pub use fs::*;
pub use info::*;
pub use view::*;
pub use view_mut::*;

/// A database tracking a set of source files
pub struct Database {
    /// The source of the files (file system or in-memory)
    source: Box<dyn FileSource>,
    /// The source code text of each file
    files: HashMap<Url, codespan::File<String>>,
    /// The AST of each file (once parsed and lowered, may be type-annotated)
    ast: HashMap<Url, Result<Arc<ast::Module>, Error>>,
    info_by_id: HashMap<Url, Lapper<u32, Info>>,
    item_by_id: HashMap<Url, Lapper<u32, Item>>,
}

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

    /// Create a new database that only keeps files in memory
    pub fn in_memory() -> Self {
        Self::from_source(InMemorySource::new())
    }

    /// Create a new database tracking the current working directory
    pub fn from_cwd() -> Self {
        Self::from_path(std::env::current_dir().expect("Could not get current directory"))
    }

    /// Create a new database with the given source
    pub fn from_source(source: impl FileSource + 'static) -> Self {
        Self {
            source: Box::new(source),
            files: HashMap::default(),
            ast: HashMap::default(),
            info_by_id: HashMap::default(),
            item_by_id: HashMap::default(),
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

    /// Open a file by its path and load it into the database
    #[cfg(not(target_arch = "wasm32"))]
    pub fn open_path<P: AsRef<std::path::Path>>(
        &mut self,
        path: P,
    ) -> Result<DatabaseViewMut<'_>, Error> {
        let path = path.as_ref().canonicalize().expect("Could not canonicalize path");
        let uri = Url::from_file_path(path).expect("Could not convert path to URI");
        self.open_url(&uri)
    }

    /// Open a file by its URI and load it into the database
    ///
    /// Returns a mutable view on the file
    pub fn open_url(&mut self, uri: &Url) -> Result<DatabaseViewMut<'_>, Error> {
        if self.source.is_modified(uri)? {
            let source = self.source.read_to_string(uri)?;
            self.files.insert(uri.clone(), codespan::File::new(uri.as_str().into(), source));
            let mut view = DatabaseViewMut { url: uri.clone(), database: self };
            let _ = view.load();
            Ok(view)
        } else {
            Ok(DatabaseViewMut { url: uri.clone(), database: self })
        }
    }

    /// Get a read-only view on a file in the database
    ///
    /// Note that this does not reload the file from the source.
    /// Only use this if you are sure the latest version is already loaded.
    /// Otherwise, use `open_url`.
    pub fn get(&self, name: &Url) -> Option<DatabaseView<'_>> {
        if self.files.contains_key(name) {
            Some(DatabaseView { url: name.clone(), database: self })
        } else {
            None
        }
    }
}
