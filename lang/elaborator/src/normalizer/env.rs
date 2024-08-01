use std::rc::Rc;

use derivative::Derivative;

use pretty::DocAllocator;
use syntax::ast::{Shift, ShiftRange};

use crate::normalizer::val::*;
use printer::tokens::COMMA;
use printer::Print;
use syntax::ast::{Idx, Lvl, Var};
use syntax::ctx::values::TypeCtx;
use syntax::ctx::{map_idx::*, GenericCtx};
use syntax::ctx::{Context, ContextElem, LevelCtx};

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Env {
    ctx: GenericCtx<Rc<Val>>,
}

impl From<GenericCtx<Rc<Val>>> for Env {
    fn from(value: GenericCtx<Rc<Val>>) -> Self {
        Env { ctx: value }
    }
}

impl Context for Env {
    type Elem = Rc<Val>;

    fn lookup<V: Into<Var>>(&self, idx: V) -> Self::Elem {
        let lvl = self.var_to_lvl(idx.into());
        self.ctx
            .bound
            .get(lvl.fst)
            .and_then(|ctx| ctx.get(lvl.snd))
            .unwrap_or_else(|| panic!("Unbound variable {lvl}"))
            .clone()
    }

    fn push_telescope(&mut self) {
        self.ctx.bound.push(vec![]);
    }

    fn pop_telescope(&mut self) {
        self.ctx.bound.pop().unwrap();
    }

    fn push_binder(&mut self, elem: Self::Elem) {
        self.ctx
            .bound
            .last_mut()
            .expect("Cannot push without calling level_inc_fst first")
            .push(elem);
    }

    fn pop_binder(&mut self, _elem: Self::Elem) {
        let err = "Cannot pop from empty context";
        self.ctx.bound.last_mut().expect(err).pop().expect(err);
    }

    fn idx_to_lvl(&self, idx: Idx) -> Lvl {
        let fst = self.ctx.bound.len() - 1 - idx.fst;
        let snd = self.ctx.bound[fst].len() - 1 - idx.snd;
        Lvl { fst, snd }
    }

    fn lvl_to_idx(&self, lvl: Lvl) -> Idx {
        let fst = self.ctx.bound.len() - 1 - lvl.fst;
        let snd = self.ctx.bound[lvl.fst].len() - 1 - lvl.snd;
        Idx { fst, snd }
    }
}

impl ContextElem<Env> for &Rc<Val> {
    fn as_element(&self) -> <Env as Context>::Elem {
        (*self).clone()
    }
}

impl Env {
    pub(super) fn map<F>(&self, f: F) -> Self
    where
        F: Fn(&Rc<Val>) -> Rc<Val>,
    {
        let bound: Vec<Vec<Rc<Val>>> =
            self.ctx.bound.iter().map(|inner| inner.iter().map(&f).collect()).collect();
        Self { ctx: bound.into() }
    }
}

impl From<Vec<Vec<Rc<Val>>>> for Env {
    fn from(bound: Vec<Vec<Rc<Val>>>) -> Self {
        Self { ctx: bound.into() }
    }
}

impl Shift for Env {
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
            .map(|(fst, v)| {
                (0..v.len())
                    .map(|snd| {
                        let idx = Idx { fst: self.bound.len() - 1 - fst, snd: v.len() - 1 - snd };
                        Rc::new(Val::Neu(Neu::Variable(Variable {
                            span: None,
                            name: String::new(),
                            idx,
                        })))
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
            .map_idx(|idx, binder| {
                Rc::new(Val::Neu(Neu::Variable(Variable {
                    // FIXME: handle info
                    span: None,
                    name: binder.name.clone(),
                    idx,
                })))
            })
            .collect();

        Env::from(bound)
    }
}

impl Print for Env {
    fn print<'a>(
        &'a self,
        cfg: &printer::PrintCfg,
        alloc: &'a printer::Alloc<'a>,
    ) -> printer::Builder<'a> {
        let iter = self.ctx.iter().map(|ctx| {
            alloc
                .intersperse(ctx.iter().map(|typ| typ.print(cfg, alloc)), alloc.text(COMMA))
                .brackets()
        });
        alloc.intersperse(iter, alloc.text(COMMA)).brackets()
    }
}
