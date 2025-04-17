use ast::{Lvl, Shift, ShiftRange, VarBound};
use pretty::DocAllocator;

use ast::ctx::LevelCtx;
use ast::ctx::map_idx::*;
use ast::ctx::values::{Binder, TypeCtx};
use ast::{Idx, Var};
use printer::Print;
use printer::tokens::COMMA;

use crate::normalizer::val::*;

#[derive(Debug, Clone)]
pub struct Env {
    /// Environment for locally bound variables
    bound_vars: Vec<Vec<Binder<Box<Val>>>>,
}

impl Env {
    pub fn lookup<V: Into<Var>>(&self, idx: V) -> Binder<Box<Val>> {
        let lvl = self.var_to_lvl(idx.into());
        self.bound_vars
            .get(lvl.fst)
            .and_then(|ctx| ctx.get(lvl.snd))
            .unwrap_or_else(|| panic!("Unbound variable {lvl}"))
            .clone()
    }

    /// Bind an iterator `iter` of binders
    pub fn bind_iter<I, O, F>(&mut self, iter: I, f: F) -> O
    where
        I: Iterator<Item = Binder<Box<Val>>>,
        F: FnOnce(&mut Self) -> O,
    {
        self.bound_vars.push(iter.collect());
        let res = f(self);
        self.bound_vars.pop().unwrap();
        res
    }
}

impl Env {
    pub fn empty() -> Self {
        Self { bound_vars: Vec::new() }
    }

    pub fn from_vec(bound: Vec<Vec<Box<Val>>>) -> Self {
        let bound_vars: Vec<Vec<_>> = bound
            .into_iter()
            .map(|inner| {
                inner
                    .into_iter()
                    .map(|v| Binder { name: ast::VarBind::Wildcard { span: None }, content: v })
                    .collect()
            })
            .collect();
        Self { bound_vars }
    }

    pub(super) fn for_each<F>(&mut self, f: F)
    where
        F: Fn(&mut Box<Val>),
    {
        for outer in self.bound_vars.iter_mut() {
            for inner in outer {
                f(&mut inner.content)
            }
        }
    }

    pub fn idx_to_lvl(&self, idx: Idx) -> Lvl {
        let fst = self.bound_vars.len() - 1 - idx.fst;
        let snd = self.bound_vars[fst].len() - 1 - idx.snd;
        Lvl { fst, snd }
    }

    pub fn lvl_to_idx(&self, lvl: Lvl) -> Idx {
        let fst = self.bound_vars.len() - 1 - lvl.fst;
        let snd = self.bound_vars[lvl.fst].len() - 1 - lvl.snd;
        Idx { fst, snd }
    }

    pub fn var_to_lvl(&self, var: Var) -> Lvl {
        match var {
            Var::Lvl(lvl) => lvl,
            Var::Idx(idx) => self.idx_to_lvl(idx),
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
                    span: None,
                    name: match &binder.name {
                        ast::VarBind::Var { span, id } => VarBound { span: *span, id: id.clone() },
                        // When we encouter a wildcard, we use `x` as a placeholder name for the variable referencing this binder.
                        // Of course, `x` is not guaranteed to be unique; in general we do not guarantee that the string representation of variables remains intact during elaboration.
                        // When reliable variable names are needed (e.g. for printing source code or code generation), the `renaming` transformation needs to be applied to the AST first.
                        ast::VarBind::Wildcard { .. } => VarBound::from_string("x"),
                    },
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
