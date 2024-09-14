use std::sync::Arc;

use url::Url;

use ast::HashMap;
use rust_lapper::Lapper;

pub use result::Error;

mod asserts;
mod info;
mod result;
mod view;
mod view_mut;

pub use info::*;
pub use view::*;
pub use view_mut::*;

#[derive(Default)]
pub struct Database {
    files: HashMap<Url, codespan::File<String>>,
    /// The AST of each file (once parsed and lowered, may be type-annotated)
    ast: HashMap<Url, Result<Arc<ast::Module>, Error>>,
    info_by_id: HashMap<Url, Lapper<u32, Info>>,
    item_by_id: HashMap<Url, Lapper<u32, Item>>,
}

/// File that can be added to the database
pub struct File {
    /// The file name or path
    pub name: Url,
    /// The source code text of the file
    pub source: String,
}

impl File {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn read(path: &std::path::Path) -> std::io::Result<Self> {
        let path = path.canonicalize()?;
        let url = Url::from_file_path(&path).map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::Other, "Cannot convert filepath to url.")
        })?;
        let file = std::fs::read_to_string(path)?;
        Ok(Self { name: url, source: file })
    }
}

impl Database {
    pub fn add(&mut self, file: File) -> DatabaseViewMut<'_> {
        let File { name, source } = file;
        self.files.insert(name.clone(), codespan::File::new(name.as_str().into(), source));
        let mut view = DatabaseViewMut { url: name, database: self };
        let _ = view.load();
        view
    }

    pub fn get(&self, name: &Url) -> Option<DatabaseView<'_>> {
        if self.files.contains_key(name) {
            Some(DatabaseView { url: name.clone(), database: self })
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, name: &Url) -> Option<DatabaseViewMut<'_>> {
        if self.files.contains_key(name) {
            Some(DatabaseViewMut { url: name.clone(), database: self })
        } else {
            None
        }
    }
}
