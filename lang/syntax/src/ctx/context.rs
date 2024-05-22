//! Variable context
//!
//! Tracks locally bound variables

use crate::{
    ast::{Idx, Leveled, Lvl, Shift, ShiftRange, Var},
    ctx::{Context, LevelCtx},
};

use super::ContextElem;

#[derive(Debug, Clone)]
pub struct GenericContext<A> {
    pub bound: Vec<Vec<A>>,
}

impl<A: Shift> GenericContext<A> {
    pub fn empty() -> Self {
        Self { bound: vec![] }
    }

    pub fn levels(&self) -> LevelCtx {
        let bound: Vec<_> = self.bound.iter().map(|inner| inner.len()).collect();
        LevelCtx::from(bound)
    }

    pub fn iter(&self) -> impl Iterator<Item = &[A]> {
        self.bound.iter().map(|inner| &inner[..])
    }

    fn shift<R: ShiftRange>(&mut self, range: R, by: (isize, isize)) {
        for lvl in 0..self.bound.len() {
            self.shift_at_lvl(range.clone(), lvl, by)
        }
    }

    fn shift_at_lvl<R: ShiftRange>(&mut self, range: R, lvl: usize, by: (isize, isize)) {
        for i in 0..self.bound[lvl].len() {
            self.bound[lvl][i] = self.bound[lvl][i].shift_in_range(range.clone(), by);
        }
    }

    pub fn len(&self) -> usize {
        self.bound.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bound.is_empty()
    }
}

impl<A: Clone + Shift> Context for GenericContext<A> {
    type ElemIn = A;

    type ElemOut = A;

    type Var = Var;

    fn lookup<V: Into<Self::Var>>(&self, idx: V) -> Self::ElemOut {
        let lvl = self.var_to_lvl(idx.into());
        self.bound
            .get(lvl.fst)
            .and_then(|ctx| ctx.get(lvl.snd))
            .unwrap_or_else(|| panic!("Unbound variable {lvl}"))
            .clone()
    }

    fn push_telescope(&mut self) {
        self.shift(0.., (1, 0));
        self.bound.push(vec![]);
    }

    fn pop_telescope(&mut self) {
        self.bound.pop().unwrap();
        self.shift(0.., (-1, 0));
    }

    fn push_binder(&mut self, elem: Self::ElemIn) {
        self.bound.last_mut().expect("Cannot push without calling level_inc_fst first").push(elem);
        self.shift_at_lvl(0..1, self.bound.len() - 1, (0, 1));
    }

    fn pop_binder(&mut self, _elem: Self::ElemIn) {
        let err = "Cannot pop from empty context";
        self.bound.last_mut().expect(err).pop().expect(err);
        self.shift_at_lvl(0..1, self.bound.len() - 1, (0, -1));
    }
}

impl<A: Clone + Shift> ContextElem<GenericContext<A>> for &A {
    fn as_element(&self) -> <GenericContext<A> as Context>::ElemIn {
        (*self).clone()
    }
}

impl<A> Leveled for GenericContext<A> {
    fn idx_to_lvl(&self, idx: Idx) -> Lvl {
        let fst = self.bound.len() - 1 - idx.fst;
        let snd = self.bound[fst].len() - 1 - idx.snd;
        Lvl { fst, snd }
    }

    fn lvl_to_idx(&self, lvl: Lvl) -> Idx {
        let fst = self.bound.len() - 1 - lvl.fst;
        let snd = self.bound[lvl.fst].len() - 1 - lvl.snd;
        Idx { fst, snd }
    }
}
