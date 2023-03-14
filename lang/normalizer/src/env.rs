use std::rc::Rc;

use derivative::Derivative;

use pretty::DocAllocator;

use crate::val::*;
use printer::tokens::COMMA;
use printer::Print;
use syntax::common::*;
use syntax::ctx::map_idx::*;
use syntax::ctx::values::TypeCtx;
use syntax::ctx::{Context, LevelCtx};

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Env {
    bound: Vec<Vec<Rc<Val>>>,
}

impl Context for Env {
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
        self.bound.push(vec![]);
    }

    fn pop_telescope(&mut self) {
        self.bound.pop().unwrap();
    }

    fn push_binder(&mut self, elem: Self::ElemIn) {
        self.bound.last_mut().expect("Cannot push without calling level_inc_fst first").push(elem);
    }

    fn pop_binder(&mut self, _elem: Self::ElemIn) {
        let err = "Cannot pop from empty context";
        self.bound.last_mut().expect(err).pop().expect(err);
    }
}

impl Leveled for Env {
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

impl Env {
    pub fn iter(&self) -> impl Iterator<Item = &[Rc<Val>]> {
        self.bound.iter().map(|inner| &inner[..])
    }

    pub(super) fn map<F>(&self, f: F) -> Self
    where
        F: Fn(&Rc<Val>) -> Rc<Val>,
    {
        let bound = self.bound.iter().map(|inner| inner.iter().map(&f).collect()).collect();
        Self { bound }
    }
}

impl From<Vec<Vec<Rc<Val>>>> for Env {
    fn from(bound: Vec<Vec<Rc<Val>>>) -> Self {
        Self { bound }
    }
}

impl ShiftInRange for Env {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        self.map(|val| val.shift_in_range(range.clone(), by))
    }
}

pub trait ToEnv {
    fn env(&self) -> Env;
}

impl ToEnv for LevelCtx {
    fn env(&self) -> Env {
        // FIXME: Refactor this
        let bound: Vec<_> = self
            .bound
            .iter()
            .enumerate()
            .map(|(fst, len)| {
                (0..*len)
                    .map(|snd| {
                        let idx = Idx { fst: self.bound.len() - 1 - fst, snd: len - 1 - snd };
                        Rc::new(Val::Neu {
                            exp: Neu::Var { info: Info::empty(), name: String::new(), idx },
                        })
                    })
                    .collect()
            })
            .collect();

        Env::from(bound)
    }
}

impl ToEnv for TypeCtx {
    fn env(&self) -> Env {
        let bound = self
            .bound
            .map_idx(|idx, _typ| {
                Rc::new(Val::Neu {
                    exp: Neu::Var {
                        // FIXME: handle info/name
                        info: Info::empty(),
                        name: String::new(),
                        idx,
                    },
                })
            })
            .collect();

        Env::from(bound)
    }
}

impl<'a> Print<'a> for Env {
    fn print(
        &'a self,
        cfg: &printer::PrintCfg,
        alloc: &'a printer::Alloc<'a>,
    ) -> printer::Builder<'a> {
        let iter = self.iter().map(|ctx| {
            alloc
                .intersperse(ctx.iter().map(|typ| typ.print(cfg, alloc)), alloc.text(COMMA))
                .brackets()
        });
        alloc.intersperse(iter, alloc.text(COMMA)).brackets()
    }
}
