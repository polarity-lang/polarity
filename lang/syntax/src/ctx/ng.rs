//! Variable context
//!
//! Tracks locally bound variables

use std::rc::Rc;

use crate::common::*;
use crate::ctx::{Context, LevelCtx};
use crate::env::Env;
use crate::values::*;

use super::map_idx::*;

#[derive(Debug, Clone)]
pub struct TypeCtx {
    bound: Vec<Vec<Rc<Val>>>,
}

impl TypeCtx {
    pub fn levels(&self) -> LevelCtx {
        let bound: Vec<_> = self.bound.iter().map(|inner| inner.len()).collect();
        LevelCtx::from(bound)
    }

    pub fn iter(&self) -> impl Iterator<Item = &[Rc<Val>]> {
        self.bound.iter().map(|inner| &inner[..])
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

impl Context for TypeCtx {
    type ElemIn = Rc<Val>;

    type ElemOut = Rc<Val>;

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
}

impl TypeCtx {
    pub fn env(&self) -> Env {
        let bound = self
            .bound
            .map_idx(|idx, typ| {
                Rc::new(Val::Neu {
                    exp: Neu::Var {
                        // FIXME: handle info/name
                        info: Info::empty(),
                        name: String::new(),
                        idx,
                    },
                    typ: typ.clone(),
                })
            })
            .collect();

        Env::from(bound)
    }
}
