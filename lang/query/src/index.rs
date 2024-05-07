use codespan::FileId;
use rust_lapper::Lapper;

use syntax::common::*;

use super::info::{Info, Item};

#[derive(Default)]
pub struct Index {
    pub(crate) index_enabled: HashSet<FileId>,
    pub(crate) info_index_by_id: HashMap<FileId, Lapper<u32, Info>>,
    pub(crate) item_index_by_id: HashMap<FileId, Lapper<u32, Item>>,
}

pub struct IndexViewMut<'a> {
    file_id: FileId,
    index: &'a mut Index,
}

pub struct IndexView<'a> {
    file_id: FileId,
    index: &'a Index,
}

impl Index {
    pub fn enable(&mut self, file_id: FileId) {
        self.index_enabled.insert(file_id);
    }

    pub fn get(&self, file_id: FileId) -> Option<IndexView<'_>> {
        self.index_enabled.get(&file_id).map(|_| IndexView { file_id, index: self })
    }

    pub fn get_mut(&mut self, file_id: FileId) -> Option<IndexViewMut<'_>> {
        if self.index_enabled.contains(&file_id) {
            Some(IndexViewMut { file_id, index: self })
        } else {
            None
        }
    }

    pub fn modify<F>(&mut self, file_id: FileId, f: F)
    where
        F: FnOnce(IndexViewMut<'_>),
    {
        if let Some(view) = self.get_mut(file_id) {
            f(view);
        }
    }
}

impl<'a> IndexViewMut<'a> {
    pub fn reset(&mut self) {
        self.index.info_index_by_id.insert(self.file_id, Lapper::new(vec![]));
        self.index.item_index_by_id.insert(self.file_id, Lapper::new(vec![]));
    }

    pub fn set(&mut self, info_index: Lapper<u32, Info>, item_index: Lapper<u32, Item>) {
        self.index.info_index_by_id.insert(self.file_id, info_index);
        self.index.item_index_by_id.insert(self.file_id, item_index);
    }
}

impl<'a> IndexView<'a> {
    pub fn infos(&self) -> &Lapper<u32, Info> {
        &self.index.info_index_by_id[&self.file_id]
    }

    pub fn items(&self) -> &Lapper<u32, Item> {
        &self.index.item_index_by_id[&self.file_id]
    }
}
