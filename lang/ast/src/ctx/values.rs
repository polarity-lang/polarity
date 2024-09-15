//! Variable context
//!
//! Tracks locally bound variables

use pretty::DocAllocator;
use printer::tokens::COMMA;
use printer::{Alloc, Builder, Print, PrintCfg};

use crate::ctx::Context;
use crate::traits::Shift;
use crate::*;

use super::{ContextElem, GenericCtx};

pub type TypeCtx = GenericCtx<Binder>;

impl TypeCtx {
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

    pub fn map_failable<E, F>(&self, f: F) -> Result<Self, E>
    where
        F: Fn(&Exp) -> Result<Box<Exp>, E>,
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

impl Context for TypeCtx {
    type Elem = Binder;

    fn lookup<V: Into<Var>>(&self, idx: V) -> Self::Elem {
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

    fn push_binder(&mut self, elem: Self::Elem) {
        self.bound.last_mut().expect("Cannot push without calling level_inc_fst first").push(elem);
        self.shift_at_lvl(0..1, self.bound.len() - 1, (0, 1));
    }

    fn pop_binder(&mut self, _elem: Self::Elem) {
        let err = "Cannot pop from empty context";
        self.bound.last_mut().expect(err).pop().expect(err);
        self.shift_at_lvl(0..1, self.bound.len() - 1, (0, -1));
    }
}

impl ContextElem<TypeCtx> for &Binder {
    fn as_element(&self) -> <TypeCtx as Context>::Elem {
        (*self).clone()
    }
}

impl Print for TypeCtx {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let iter = self.iter().map(|ctx| {
            alloc
                .intersperse(ctx.iter().map(|b| b.typ.print(cfg, alloc)), alloc.text(COMMA))
                .brackets()
        });
        alloc.intersperse(iter, alloc.text(COMMA)).brackets()
    }
}

#[derive(Debug, Clone)]
pub struct Binder {
    pub name: Ident,
    pub typ: Box<Exp>,
}

impl Shift for Binder {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        Self { name: self.name.clone(), typ: self.typ.shift_in_range(range, by) }
    }
}
