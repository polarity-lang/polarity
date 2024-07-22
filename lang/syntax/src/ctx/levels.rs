use std::fmt;

// use data::string::comma_separated;
fn comma_separated<I: IntoIterator<Item = String>>(iter: I) -> String {
    separated(", ", iter)
}
fn separated<I: IntoIterator<Item = String>>(s: &str, iter: I) -> String {
    let vec: Vec<_> = iter.into_iter().collect();
    vec.join(s)
}

use crate::ast::*;

use super::def::*;

#[derive(Clone, Debug, Default)]
pub struct LevelCtx {
    /// Number of binders in the second dimension for each first dimension
    pub bound: Vec<usize>,
}

impl LevelCtx {
    pub fn empty() -> Self {
        Self { bound: vec![] }
    }

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

    // Swap the given indices
    pub fn swap(&self, fst1: usize, fst2: usize) -> Self {
        let mut new_ctx = self.clone();
        new_ctx.bound.swap(fst1, fst2);
        new_ctx
    }
}

impl Context for LevelCtx {
    type Elem = ();

    fn push_telescope(&mut self) {
        self.bound.push(0);
    }

    fn pop_telescope(&mut self) {
        self.bound.pop().unwrap();
    }

    fn push_binder(&mut self, _elem: Self::Elem) {
        *self.bound.last_mut().expect("Cannot push without calling level_inc_fst first") += 1;
    }

    fn pop_binder(&mut self, _elem: Self::Elem) {
        let err = "Cannot pop from empty context";
        *self.bound.last_mut().expect(err) -= 1;
    }

    fn lookup<V: Into<Var>>(&self, _idx: V) -> Self::Elem {
        ()
    }

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

impl<T> ContextElem<LevelCtx> for T {
    fn as_element(&self) -> <LevelCtx as Context>::Elem {}
}

impl From<Vec<usize>> for LevelCtx {
    fn from(bound: Vec<usize>) -> Self {
        Self { bound }
    }
}

impl fmt::Display for LevelCtx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}]", comma_separated(self.bound.iter().map(ToString::to_string)))
    }
}
