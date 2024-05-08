use codespan::FileId;

mod edit;
mod lift;
mod rt;
mod spans;
mod xfunc;

pub use self::xfunc::*;
pub use edit::*;

use crate::*;

/// View on a file in the database
pub struct DatabaseView<'a> {
    pub(crate) file_id: FileId,
    pub(crate) database: &'a Database,
}

impl<'a> DatabaseView<'a> {
    pub fn index(&self) -> IndexView<'_> {
        let DatabaseView { file_id, database } = self;
        database.index.get(*file_id)
    }
}
