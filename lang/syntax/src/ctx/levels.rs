use std::fmt;

use data::string::comma_separated;

use crate::common::*;

use super::def::*;

#[derive(Clone, Debug, Default)]
pub struct LevelCtx {
    /// Number of binders in the second dimension for each first dimension
    pub bound: Vec<usize>,
}

impl LevelCtx {
    pub fn is_empty(&self) -> bool {
        self.bound.is_empty()
    }

    pub fn len(&self) -> usize {
        self.bound.len()
    }

    pub fn append(&self, other: &LevelCtx) -> Self {
        let mut bound = self.bound.clone();
        bound.extend(other.bound.iter().cloned());
        Self { bound }
    }

    pub fn tail(&self, skip: usize) -> Self {
        Self { bound: self.bound.iter().skip(skip).cloned().collect() }
    }
}

impl Context for LevelCtx {
    type ElemIn = ();
    type ElemOut = usize;

    type Var = usize;

    fn empty() -> Self {
        Self { bound: vec![] }
    }

    fn push_telescope(&mut self) {
        self.bound.push(0);
    }

    fn pop_telescope(&mut self) {
        self.bound.pop().unwrap();
    }

    fn push_binder(&mut self, _elem: Self::ElemIn) {
        *self.bound.last_mut().expect("Cannot push without calling level_inc_fst first") += 1;
    }

    fn pop_binder(&mut self, _elem: Self::ElemIn) {
        let err = "Cannot pop from empty context";
        *self.bound.last_mut().expect(err) -= 1;
    }

    fn lookup<V: Into<Self::Var>>(&self, idx: V) -> Self::ElemOut {
        self.bound[idx.into()]
    }
}

impl<T> ContextElem<LevelCtx> for T {
    fn as_element(&self) -> <LevelCtx as Context>::ElemIn {}
}

impl From<Vec<usize>> for LevelCtx {
    fn from(bound: Vec<usize>) -> Self {
        Self { bound }
    }
}

impl Leveled for LevelCtx {
    fn idx_to_lvl(&self, idx: Idx) -> Lvl {
        let fst = self.bound.len() - 1 - idx.fst;
        let snd = self.bound[fst] - 1 - idx.snd;
        Lvl { fst, snd }
    }

    fn lvl_to_idx(&self, lvl: Lvl) -> Idx {
        let fst = self.bound.len() - 1 - lvl.fst;
        let snd = self.bound[lvl.fst] - 1 - lvl.snd;
        Idx { fst, snd }
    }
}

impl Swap for LevelCtx {
    fn swap(&self, fst1: usize, fst2: usize) -> Self {
        let mut new_ctx = self.clone();
        new_ctx.bound.swap(fst1, fst2);
        new_ctx
    }
}

impl fmt::Display for LevelCtx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}]", comma_separated(self.bound.iter().map(ToString::to_string)))
    }
}
