use crate::{
    HashSet, Lvl,
    ctx::{LevelCtx, values::Binder},
};

pub trait FreeVars {
    /// Set of free variables that occur syntactically in the expression.
    ///
    /// This is not the same as the free variables closure, which would also includes variables
    /// occuring in the types of the free variables, recursively.
    ///
    /// Parameters:
    ///
    /// - `ctx`: The context in which the free variables are computed.
    ///   Any free variables computed will be bound in this context.
    /// - `cutoff`: The cutoff (de Bruijn index). Any variable with an index less than `cutoff` is considered bound.
    ///   Alternatively, you can think of `cutoff` tracking the number of de Bruijn levels that are bound after `ctx`.
    fn free_vars(&self, ctx: &LevelCtx, cutoff: usize) -> HashSet<Lvl>;
}

impl<T: FreeVars> FreeVars for Option<T> {
    fn free_vars(&self, ctx: &LevelCtx, cutoff: usize) -> HashSet<Lvl> {
        match self {
            Some(exp) => exp.free_vars(ctx, cutoff),
            None => HashSet::default(),
        }
    }
}

impl<T: FreeVars> FreeVars for Vec<T> {
    fn free_vars(&self, ctx: &LevelCtx, cutoff: usize) -> HashSet<Lvl> {
        self.iter().flat_map(|exp| exp.free_vars(ctx, cutoff)).collect()
    }
}

impl<T: FreeVars> FreeVars for Box<T> {
    fn free_vars(&self, ctx: &LevelCtx, cutoff: usize) -> HashSet<Lvl> {
        self.as_ref().free_vars(ctx, cutoff)
    }
}

impl<T: FreeVars> FreeVars for Binder<T> {
    fn free_vars(&self, ctx: &LevelCtx, cutoff: usize) -> HashSet<Lvl> {
        self.content.free_vars(ctx, cutoff)
    }
}
