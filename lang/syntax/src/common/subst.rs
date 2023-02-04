use crate::common::*;
use crate::ctx::*;

pub struct Assign<K, V>(pub K, pub V);

pub trait Substitution<E>: ShiftInRange {
    fn get_subst(&self, ctx: &LevelCtx, lvl: Lvl) -> Option<E>;
}

impl<E: Clone + ShiftInRange> Substitution<E> for Vec<Vec<E>> {
    fn get_subst(&self, _ctx: &LevelCtx, lvl: Lvl) -> Option<E> {
        if lvl.fst >= self.len() {
            return None;
        }
        Some(self[lvl.fst][lvl.snd].clone())
    }
}

impl<K: Clone, V: ShiftInRange> ShiftInRange for Assign<K, V> {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        Assign(self.0.clone(), self.1.shift_in_range(range, by))
    }
}

impl<E: Clone + ShiftInRange> Substitution<E> for Assign<Lvl, E> {
    fn get_subst(&self, _ctx: &LevelCtx, lvl: Lvl) -> Option<E> {
        if self.0 == lvl {
            Some(self.1.clone())
        } else {
            None
        }
    }
}

pub trait Substitutable<E>: Sized {
    fn subst<S: Substitution<E>>(&self, ctx: &mut LevelCtx, by: &S) -> Self;
}

pub trait SubstTelescope<E> {
    fn subst_telescope<S: Substitution<E>>(&self, lvl: Lvl, by: &S) -> Self;
}

impl<E, T: Substitutable<E>> Substitutable<E> for Option<T> {
    fn subst<S: Substitution<E>>(&self, ctx: &mut LevelCtx, by: &S) -> Self {
        self.as_ref().map(|x| x.subst(ctx, by))
    }
}

impl<E, T: Substitutable<E>> Substitutable<E> for Vec<T> {
    fn subst<S: Substitution<E>>(&self, ctx: &mut LevelCtx, by: &S) -> Self {
        self.iter().map(|x| x.subst(ctx, by)).collect()
    }
}

// Swap the given indices
pub trait Swap {
    fn swap(&self, fst1: usize, fst2: usize) -> Self;
}

pub trait SwapWithCtx<E> {
    fn swap_with_ctx(&self, ctx: &mut LevelCtx, fst1: usize, fst2: usize) -> Self;
}

impl<E> Substitutable<E> for () {
    fn subst<S: Substitution<E>>(&self, _ctx: &mut LevelCtx, _by: &S) -> Self {}
}
