use crate::{
    HashSet, Lvl,
    ctx::{LevelCtx, values::Binder},
};

pub trait FreeVars {
    /// Helper function to compute the set of free variables by mutably adding free variables
    /// to a HashSet of variables.
    ///
    /// Parameters:
    ///
    /// - `ctx`: The context in which the free variables are computed.
    ///   Any free variables computed will be bound in this context.
    /// - `cutoff`: The cutoff (de Bruijn index). Any variable with an index less than `cutoff` is considered bound.
    ///   Alternatively, you can think of `cutoff` tracking the number of de Bruijn levels that are bound after `ctx`.
    /// - `fvs`: The (partially computed) set of free variables occurring in the expression.
    ///
    fn free_vars_mut(&self, ctx: &LevelCtx, cutoff: usize, fvs: &mut HashSet<Lvl>);

    /// Set of free variables that occur syntactically in the expression.
    ///
    /// This is not the same as the free variables closure, which would also includes variables
    /// occuring in the types of the free variables, recursively.
    ///
    /// Parameters:
    ///
    /// - `ctx`: The context in which the free variables are computed.
    ///   Any free variables computed will be bound in this context.
    fn free_vars(&self, ctx: &LevelCtx) -> HashSet<Lvl> {
        let mut fvs: HashSet<Lvl> = HashSet::default();
        self.free_vars_mut(ctx, 0, &mut fvs);
        fvs
    }
}

impl<T: FreeVars> FreeVars for Option<T> {
    fn free_vars_mut(&self, ctx: &LevelCtx, cutoff: usize, fvs: &mut HashSet<Lvl>) {
        if let Some(exp) = self {
            exp.free_vars_mut(ctx, cutoff, fvs)
        }
    }
}

impl<T: FreeVars> FreeVars for Vec<T> {
    fn free_vars_mut(&self, ctx: &LevelCtx, cutoff: usize, fvs: &mut HashSet<Lvl>) {
        for i in self {
            i.free_vars_mut(ctx, cutoff, fvs)
        }
    }
}

impl<T: FreeVars> FreeVars for Box<T> {
    fn free_vars_mut(&self, ctx: &LevelCtx, cutoff: usize, fvs: &mut HashSet<Lvl>) {
        self.as_ref().free_vars_mut(ctx, cutoff, fvs)
    }
}

impl<T: FreeVars> FreeVars for Binder<T> {
    fn free_vars_mut(&self, ctx: &LevelCtx, cutoff: usize, fvs: &mut HashSet<Lvl>) {
        self.content.free_vars_mut(ctx, cutoff, fvs)
    }
}
