//! Variable context
//!
//! Tracks locally bound variables

use std::rc::Rc;

use crate::common::*;
use crate::ctx::{Context, LevelCtx};
use crate::nf::*;

use super::ContextElem;

#[derive(Debug, Clone)]
pub struct TypeCtx {
    pub bound: Vec<Vec<Binder>>,
}

impl TypeCtx {
    pub fn levels(&self) -> LevelCtx {
        let bound: Vec<_> = self.bound.iter().map(|inner| inner.len()).collect();
        LevelCtx::from(bound)
    }

    pub fn iter(&self) -> impl Iterator<Item = &[Binder]> {
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
}

impl Context for TypeCtx {
    type ElemIn = Binder;

    type ElemOut = Binder;

    type Var = Var;

    fn empty() -> Self {
        Self { bound: vec![] }
    }

    fn lookup<V: Into<Self::Var>>(&self, idx: V) -> Self::ElemOut {
        let lvl = self.var_to_lvl(idx.into());
        self.bound
            .get(lvl.fst)
            .and_then(|ctx| ctx.get(lvl.snd))
            .unwrap_or_else(|| panic!("Unbound variable {lvl}"))
            .clone()
    }

    fn push_telescope(&mut self) {
        self.shift(.., (1, 0));
        self.bound.push(vec![]);
    }

    fn pop_telescope(&mut self) {
        self.bound.pop().unwrap();
        self.shift(.., (-1, 0));
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

impl ContextElem<TypeCtx> for &Binder {
    fn as_element(&self) -> <TypeCtx as Context>::ElemIn {
        (*self).clone()
    }
}

impl Leveled for TypeCtx {
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

impl TypeCtx {
    pub fn len(&self) -> usize {
        self.bound.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bound.is_empty()
    }

    pub fn map_failable<E, F>(&self, f: F) -> Result<Self, E>
    where
        F: Fn(&Rc<Nf>) -> Result<Rc<Nf>, E>,
    {
        let bound: Result<_, _> = self
            .bound
            .iter()
            .map(|stack| {
                stack.iter().map(|b| Ok(Binder { name: b.name.clone(), typ: f(&b.typ)? })).collect()
            })
            .collect();

        Ok(Self { bound: bound? })
    }
}

#[derive(Debug, Clone)]
pub struct Binder {
    pub name: Ident,
    pub typ: Rc<Nf>,
}

impl ShiftInRange for Binder {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        Self { name: self.name.clone(), typ: self.typ.shift_in_range(range, by) }
    }
}
