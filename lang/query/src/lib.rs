use std::sync::Arc;

use parser::cst;
use url::Url;

use ast::HashMap;
use rust_lapper::Lapper;

pub use result::Error;

mod asserts;
mod edit;
mod fs;
mod info;
mod lift;
mod modules;
mod result;
mod rt;
mod spans;
mod view_mut;
mod xfunc;

pub use edit::*;
pub use fs::*;
pub use info::*;
pub use view_mut::*;
pub use xfunc::*;

/// A database tracking a set of source files
pub struct Database {
    /// The source of the files (file system or in-memory)
    source: Box<dyn FileSource>,
    /// The source code text of each file
    files: HashMap<Url, codespan::File<String>>,
    /// The CST of each file (once parsed)
    cst: HashMap<Url, Result<Arc<cst::decls::Module>, Error>>,
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
            cst: HashMap::default(),
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
        self.open_uri(&uri)
    }

    /// Open a file by its URI and load the source into the database
    ///
    /// Returns a mutable view on the file
    pub fn open_uri(&mut self, uri: &Url) -> Result<DatabaseViewMut<'_>, Error> {
        if self.source.is_modified(uri)? {
            let source = self.source.read_to_string(uri)?;
            self.files.insert(uri.clone(), codespan::File::new(uri.as_str().into(), source));
            Ok(DatabaseViewMut { url: uri.clone(), database: self })
        } else {
            Ok(DatabaseViewMut { url: uri.clone(), database: self })
        }
    }
}
