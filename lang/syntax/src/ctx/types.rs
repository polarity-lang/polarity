//! Variable context
//!
//! Tracks locally bound variables

use std::rc::Rc;

use crate::ast::*;
use crate::ast::{subst, Substitutable};
use crate::common::*;
use crate::ctx::{Context, LevelCtx};

#[derive(Debug, Clone)]
pub struct TypeCtx<P: Phase> {
    bound: Vec<Vec<Rc<Exp<P>>>>,
}

impl<P: Phase> TypeCtx<P>
where
    P::Typ: ShiftCutoff,
{
    pub fn levels(&self) -> LevelCtx {
        let bound: Vec<_> = self.bound.iter().map(|inner| inner.len()).collect();
        LevelCtx::from(bound)
    }

    pub fn iter(&self) -> impl Iterator<Item = &[Rc<Exp<P>>]> {
        self.bound.iter().map(|inner| &inner[..])
    }

    fn map<F>(&self, f: F) -> Self
    where
        F: Fn(&Rc<Exp<P>>) -> Rc<Exp<P>>,
    {
        let bound = self.bound.iter().map(|inner| inner.iter().map(&f).collect()).collect();
        Self { bound }
    }

    fn shift(&mut self, by: (isize, isize)) {
        for lvl in 0..self.bound.len() {
            self.shift_at_lvl(lvl, by)
        }
    }

    fn shift_at_lvl(&mut self, lvl: usize, by: (isize, isize)) {
        for i in 0..self.bound[lvl].len() {
            self.bound[lvl][i] = self.bound[lvl][i].shift(by);
        }
    }
}

impl<P: Phase> Context for TypeCtx<P>
where
    P::Typ: ShiftCutoff,
{
    type ElemIn = Rc<Exp<P>>;

    type ElemOut = Rc<Exp<P>>;

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
        self.shift((1, 0));
        self.bound.push(vec![]);
    }

    fn pop_telescope(&mut self) {
        self.bound.pop().unwrap();
        self.shift((-1, 0));
    }

    fn push_binder(&mut self, elem: Self::ElemIn) {
        self.bound.last_mut().expect("Cannot push without calling level_inc_fst first").push(elem);
        self.shift_at_lvl(self.bound.len() - 1, (0, 1));
    }

    fn pop_binder(&mut self, _elem: Self::ElemIn) {
        let err = "Cannot pop from empty context";
        self.bound.last_mut().expect(err).pop().expect(err);
        self.shift_at_lvl(self.bound.len() - 1, (0, -1));
    }
}

impl<P: Phase> Leveled for TypeCtx<P> {
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

impl<P: Phase> Substitutable<P> for TypeCtx<P>
where
    P::Typ: Substitutable<P> + ShiftCutoff,
{
    fn subst<S: Substitution<P>>(&self, _ctx: &mut subst::Ctx, by: &S) -> Self {
        self.map(|exp| exp.subst(&mut self.levels(), by))
    }
}

impl<P: Phase> TypeCtx<P> {
    pub fn len(&self) -> usize {
        self.bound.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bound.is_empty()
    }
}
