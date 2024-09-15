use std::rc::Rc;

use crate::*;
use url::Url;

/// Mutable view on a file in the database
pub struct DatabaseViewMut<'a> {
    pub(crate) url: Url,
    pub(crate) database: &'a mut Database,
}

impl<'a> DatabaseViewMut<'a> {
    pub fn load(&mut self) -> Result<(), Error> {
        self.reset();
        let prg = self.query_ref().tst()?;
        let (info_lapper, item_lapper) = collect_info(Rc::new(prg));
        self.set(info_lapper, item_lapper);
        Ok(())
    }

    pub fn update(&mut self, source: String) -> Result<(), Error> {
        let DatabaseViewMut { url, database } = self;
        let mut file = database.files.remove(url).unwrap();
        file.update(source);
        database.files.insert(url.clone(), file);
        self.load()
    }

    pub fn query(self) -> DatabaseView<'a> {
        let DatabaseViewMut { url, database } = self;
        DatabaseView { url, database }
    }

    pub fn query_ref(&self) -> DatabaseView<'_> {
        let DatabaseViewMut { url, database } = self;
        DatabaseView { url: url.clone(), database }
    }

    pub fn reset(&mut self) {
        self.database.info_by_id.insert(self.url.clone(), Lapper::new(vec![]));
        self.database.item_by_id.insert(self.url.clone(), Lapper::new(vec![]));
    }

    pub fn set(&mut self, info_index: Lapper<u32, Info>, item_index: Lapper<u32, Item>) {
        self.database.info_by_id.insert(self.url.clone(), info_index);
        self.database.item_by_id.insert(self.url.clone(), item_index);
    }
}
