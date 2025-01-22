use ast::{Shift, ShiftRange, VarBound};
use pretty::DocAllocator;

use ast::ctx::values::{Binder, TypeCtx};
use ast::ctx::{map_idx::*, GenericCtx};
use ast::ctx::{Context, ContextElem, LevelCtx};
use ast::{Idx, Var};
use printer::tokens::COMMA;
use printer::Print;

use crate::normalizer::val::*;

#[derive(Debug, Clone)]
pub struct Env {
    /// Environment for locally bound variables
    bound_vars: GenericCtx<Box<Val>>,
}

impl Context for Env {
    type Elem = Binder<Box<Val>>;

    fn lookup<V: Into<Var>>(&self, idx: V) -> Self::Elem {
        let lvl = self.bound_vars.var_to_lvl(idx.into());
        self.bound_vars
            .bound
            .get(lvl.fst)
            .and_then(|ctx| ctx.get(lvl.snd))
            .unwrap_or_else(|| panic!("Unbound variable {lvl}"))
            .clone()
    }

    fn push_telescope(&mut self) {
        self.bound_vars.bound.push(vec![]);
    }

    fn pop_telescope(&mut self) {
        self.bound_vars.bound.pop().unwrap();
    }

    fn push_binder(&mut self, elem: Self::Elem) {
        self.bound_vars
            .bound
            .last_mut()
            .expect("Cannot push without calling push_telescope first")
            .push(elem);
    }

    fn pop_binder(&mut self, _elem: Self::Elem) {
        let err = "Cannot pop from empty context";
        self.bound_vars.bound.last_mut().expect(err).pop().expect(err);
    }
}

impl ContextElem<Env> for Binder<Box<Val>> {
    fn as_element(&self) -> <Env as Context>::Elem {
        (*self).clone()
    }
}

impl Env {
    pub fn empty() -> Self {
        Self { bound_vars: GenericCtx::empty() }
    }

    pub fn from_vec(bound: Vec<Vec<Box<Val>>>) -> Self {
        let bound: Vec<Vec<_>> = bound
            .into_iter()
            .map(|inner| {
                inner
                    .into_iter()
                    .map(|v| Binder { name: ast::VarBind::Wildcard { span: None }, content: v })
                    .collect()
            })
            .collect();
        Self { bound_vars: GenericCtx::from(bound) }
    }

    pub(super) fn for_each<F>(&mut self, f: F)
    where
        F: Fn(&mut Box<Val>),
    {
        for outer in self.bound_vars.bound.iter_mut() {
            for inner in outer {
                f(&mut inner.content)
            }
        }
    }
}

impl Shift for Env {
    fn shift_in_range<R: ShiftRange>(&mut self, range: &R, by: (isize, isize)) {
        self.for_each(|val| val.shift_in_range(range, by))
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
                        Box::new(Val::Neu(Neu::Variable(Variable {
                            span: None,
                            name: VarBound::from_string(""),
                            idx,
                        })))
                    })
                    .collect()
            })
            .collect();

        Env::from_vec(bound)
    }
}

impl ToEnv for TypeCtx {
    fn env(&self) -> Env {
        let bound = self
            .bound
            .map_idx(|idx, binder| {
                Box::new(Val::Neu(Neu::Variable(Variable {
                    // FIXME: handle info
                    span: None,
                    name: binder.name.clone().into(),
                    idx,
                })))
            })
            .collect();

        Env::from_vec(bound)
    }
}

impl Print for Env {
    fn print<'a>(
        &'a self,
        cfg: &printer::PrintCfg,
        alloc: &'a printer::Alloc<'a>,
    ) -> printer::Builder<'a> {
        let iter = self.bound_vars.iter().map(|ctx| {
            alloc
                .intersperse(ctx.iter().map(|v| v.content.print(cfg, alloc)), alloc.text(COMMA))
                .brackets()
        });
        alloc.intersperse(iter, alloc.text(COMMA)).brackets()
    }
}
