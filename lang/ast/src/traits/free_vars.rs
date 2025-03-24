use std::hash::Hash;

use crate::{
    ctx::{
        values::{Binder, TypeCtx},
        LevelCtx,
    },
    HashSet, Lvl,
};

/// Compute the set of free variables (either syntactic or closed under type dependencies)
pub trait FreeVars {
    /// Compute the set of free variables that occur in an expression, closed under type dependencies
    fn free_vars_closure(&self, ctx: &TypeCtx) -> HashSet<Lvl> {
        let mut level_ctx = ctx.levels();
        let mut fvs = self.free_vars(&mut level_ctx, ctx.cutoff());

        for (fst, telescope) in ctx.bound.iter().rev().enumerate() {
            for (snd, binder) in telescope.iter().rev().enumerate() {
                let lvl = Lvl { fst, snd };
                if fvs.contains(&lvl) {
                    fvs.extend(binder.content.free_vars(&mut level_ctx, lvl));
                }
            }
        }

        fvs
    }

    /// Compute the set of free variables that syntactically occur in an expression
    ///
    /// For example, under `Γ = [x: Bool, y: BoolRep(x)]`, `free_vars(y) = { y }`.
    /// Even though the type of `y` depends on `x`, since `x` does not syntactically occur in the expression, it is not included in the set.
    /// For many purposes, we are actually interested in the free variables closed under type dependencies.
    /// For this, see `free_vars_closure`.
    ///
    /// # Parameters
    ///
    /// - `ctx`: The context of the expression
    /// - `cutoff`: Only variables with `level < cutoff` are included.
    ///    For example, if `Γ = []`, the cutoff would be `(0, 0)`, and no variables would be included.
    ///    If `Γ = [[x: Bool]]`, the cutoff would be `(0, 1)`, and only `x` (which has level `(0, 0)`) would be included.
    fn free_vars(&self, ctx: &mut LevelCtx, cutoff: Lvl) -> HashSet<Lvl>;
}

impl<T: FreeVars> FreeVars for Vec<T> {
    fn free_vars(&self, ctx: &mut LevelCtx, cutoff: Lvl) -> HashSet<Lvl> {
        self.iter().flat_map(|x| x.free_vars(ctx, cutoff)).collect()
    }
}

impl<T: FreeVars> FreeVars for Option<T> {
    fn free_vars(&self, ctx: &mut LevelCtx, cutoff: Lvl) -> HashSet<Lvl> {
        self.as_ref().map_or_else(HashSet::default, |x| x.free_vars(ctx, cutoff))
    }
}

impl<T: FreeVars> FreeVars for Box<T> {
    fn free_vars(&self, ctx: &mut LevelCtx, cutoff: Lvl) -> HashSet<Lvl> {
        self.as_ref().free_vars(ctx, cutoff)
    }
}

impl<T: FreeVars> FreeVars for Binder<T> {
    fn free_vars(&self, ctx: &mut LevelCtx, cutoff: Lvl) -> HashSet<Lvl> {
        let Binder { name: _, content } = self;
        content.free_vars(ctx, cutoff)
    }
}
