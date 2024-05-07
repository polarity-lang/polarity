use codespan::FileId;
use rust_lapper::Lapper;

use syntax::common::*;

use super::info::{HoverInfo, Item};

#[derive(Default)]
pub struct Index {
    pub(crate) info_index_by_id: HashMap<FileId, Lapper<u32, HoverInfo>>,
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
    pub fn get(&self, file_id: FileId) -> IndexView<'_> {
        IndexView { file_id, index: self }
    }

    pub fn get_mut(&mut self, file_id: FileId) -> IndexViewMut<'_> {
        IndexViewMut { file_id, index: self }
    }

    pub fn modify<F>(&mut self, file_id: FileId, f: F)
    where
        F: FnOnce(IndexViewMut<'_>),
    {
        let view = self.get_mut(file_id);
        f(view);
    }
}

impl<'a> IndexViewMut<'a> {
    pub fn reset(&mut self) {
        self.index.info_index_by_id.insert(self.file_id, Lapper::new(vec![]));
        self.index.item_index_by_id.insert(self.file_id, Lapper::new(vec![]));
    }

    pub fn set(&mut self, info_index: Lapper<u32, HoverInfo>, item_index: Lapper<u32, Item>) {
        self.index.info_index_by_id.insert(self.file_id, info_index);
        self.index.item_index_by_id.insert(self.file_id, item_index);
    }
}

impl<'a> IndexView<'a> {
    pub fn infos(&self) -> &Lapper<u32, HoverInfo> {
        &self.index.info_index_by_id[&self.file_id]
    }

    pub fn items(&self) -> &Lapper<u32, Item> {
        &self.index.item_index_by_id[&self.file_id]
    }
}
