use codespan::FileId;

use crate::*;

/// Mutable view on a file in the database
pub struct DatabaseViewMut<'a> {
    pub(crate) file_id: FileId,
    pub(crate) database: &'a mut Database,
}

impl<'a> DatabaseViewMut<'a> {
    pub fn load(&mut self) -> Result<(), Error> {
        self.reset();
        let prg = self.query_ref().tst()?;
        let (info_lapper, item_lapper) = collect_info(&prg);
        self.set(info_lapper, item_lapper);
        Ok(())
    }

    pub fn update(&mut self, source: String) -> Result<(), Error> {
        let DatabaseViewMut { file_id, database } = self;
        database.files.update(*file_id, source);
        self.load()
    }

    pub fn query(self) -> DatabaseView<'a> {
        let DatabaseViewMut { file_id, database } = self;
        DatabaseView { file_id, database }
    }

    pub fn query_ref(&self) -> DatabaseView<'_> {
        let DatabaseViewMut { file_id, database } = self;
        DatabaseView { file_id: *file_id, database }
    }

    pub fn reset(&mut self) {
        self.database.info_by_id.insert(self.file_id, Lapper::new(vec![]));
        self.database.item_by_id.insert(self.file_id, Lapper::new(vec![]));
    }

    pub fn set(&mut self, info_index: Lapper<u32, Info>, item_index: Lapper<u32, Item>) {
        self.database.info_by_id.insert(self.file_id, info_index);
        self.database.item_by_id.insert(self.file_id, item_index);
    }
}
