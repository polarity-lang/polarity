use std::rc::Rc;

use crate::ast::Exp;
use crate::common::*;
use crate::ctx::*;

pub struct Assign<K, V>(pub K, pub V);

pub trait Substitution: Shift {
    fn get_subst(&self, ctx: &LevelCtx, lvl: Lvl) -> Option<Rc<Exp>>;
}

impl Substitution for Vec<Vec<Rc<Exp>>> {
    fn get_subst(&self, _ctx: &LevelCtx, lvl: Lvl) -> Option<Rc<Exp>> {
        if lvl.fst >= self.len() {
            return None;
        }
        Some(self[lvl.fst][lvl.snd].clone())
    }
}

impl<K: Clone, V: Shift> Shift for Assign<K, V> {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        Assign(self.0.clone(), self.1.shift_in_range(range, by))
    }
}

impl Substitution for Assign<Lvl, Rc<Exp>> {
    fn get_subst(&self, _ctx: &LevelCtx, lvl: Lvl) -> Option<Rc<Exp>> {
        if self.0 == lvl {
            Some(self.1.clone())
        } else {
            None
        }
    }
}

pub trait Substitutable: Sized {
    fn subst<S: Substitution>(&self, ctx: &mut LevelCtx, by: &S) -> Self;
}

pub trait SubstTelescope {
    fn subst_telescope<S: Substitution>(&self, lvl: Lvl, by: &S) -> Self;
}

impl<T: Substitutable> Substitutable for Option<T> {
    fn subst<S: Substitution>(&self, ctx: &mut LevelCtx, by: &S) -> Self {
        self.as_ref().map(|x| x.subst(ctx, by))
    }
}

impl<T: Substitutable> Substitutable for Vec<T> {
    fn subst<S: Substitution>(&self, ctx: &mut LevelCtx, by: &S) -> Self {
        self.iter().map(|x| x.subst(ctx, by)).collect()
    }
}

// Swap the given indices
pub trait Swap {
    fn swap(&self, fst1: usize, fst2: usize) -> Self;
}

pub trait SwapWithCtx {
    fn swap_with_ctx(&self, ctx: &mut LevelCtx, fst1: usize, fst2: usize) -> Self;
}
