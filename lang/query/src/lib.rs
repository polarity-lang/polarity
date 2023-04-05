use std::fs;
use std::io;
use std::path::Path;

use codespan::{FileId, Files};

use data::HashMap;

use info::collect_info;
pub use result::Error;

mod asserts;
mod index;
mod info;
mod result;
mod view;
mod view_mut;

pub use index::*;
pub use info::*;
pub use view::*;
pub use view_mut::*;

#[derive(Default)]
pub struct Database {
    id_by_name: HashMap<String, FileId>,
    files: Files<String>,
    index: index::Index,
}

/// File that can be added to the database
#[derive(Default)]
pub struct File {
    /// The file name or path
    pub name: String,
    /// The source code text of the file
    pub source: String,
    /// Whether to index this file
    pub index: bool,
}

impl File {
    pub fn read(path: &Path) -> io::Result<Self> {
        Ok(Self {
            name: path.to_str().unwrap().to_owned(),
            source: fs::read_to_string(path)?,
            index: false,
        })
    }
}

impl Database {
    pub fn add(&mut self, file: File) -> DatabaseViewMut<'_> {
        let File { name, source, index } = file;
        let file_id = self.files.add(name.clone(), source);
        self.id_by_name.insert(name, file_id);
        if index {
            self.index.enable(file_id);
        }
        DatabaseViewMut { file_id, database: self }
    }

    pub fn get(&self, name: &str) -> Option<DatabaseView<'_>> {
        self.id_by_name.get(name).map(|file_id| DatabaseView { file_id: *file_id, database: self })
    }

    pub fn get_mut(&mut self, name: &str) -> Option<DatabaseViewMut<'_>> {
        // HACK: Replacing this by `Option::map` does not compile
        // (as of Rust 1.64.0, clippy 0.1.64)
        #[allow(clippy::manual_map)]
        match self.id_by_name.get(name) {
            Some(file_id) => Some(DatabaseViewMut { file_id: *file_id, database: self }),
            None => None,
        }
    }
}
