use crate::ctx::LevelCtx;
use crate::exp::Exp;
use crate::{Hole, Idx, Lvl, MetaVar, Variable};

/// Whether a subexpression that fulfills a predicate occurs
///
/// The actual check is done in the implementation for [Exp].
/// The other implementations just pass the call to all subexpressions.
pub trait Occurs {
    /// Whether a subexpression that fulfills a predicate occurs
    ///
    /// # Parameters
    ///
    /// - `ctx`: current context under which `self` is closed
    /// - `f`: the predicate which is called on all subexpressions
    ///
    /// # Returns
    ///
    /// Whether the predicate `f` evaluates to `true` on any subexpression
    fn occurs<F>(&self, ctx: &mut LevelCtx, f: &F) -> bool
    where
        F: Fn(&LevelCtx, &Exp) -> bool;
    /// Whether a variable with the given De-Bruijn level occurs as a subexpression
    ///
    /// # Parameters
    ///
    /// - `ctx`: current context under which `self` is closed
    /// - `lvl`: De-Bruijn level to search for
    ///
    /// # Returns
    ///
    /// Whether a variable with the given De-Bruijn level occurs in the expression.
    fn occurs_var(&self, ctx: &mut LevelCtx, lvl: Lvl) -> bool {
        self.occurs(ctx, &|ctx, exp| match exp {
            Exp::Variable(Variable { idx, .. }) => ctx.idx_to_lvl(*idx) == lvl,
            _ => false,
        })
    }
    /// Whether a metavariable with the given `meta_var_id` occurs as a subexpression
    ///
    /// # Parameters
    ///
    /// - `ctx`: current context under which `self` is closed
    /// - `metavar`: the metavariable we are looking for
    ///
    /// # Returns
    ///
    /// Whether a hole with `meta_var_id` occurs as a subexpression
    fn occurs_metavar(&self, ctx: &mut LevelCtx, metavar: &MetaVar) -> bool {
        let meta_var_id = metavar.id;
        self.occurs(ctx, &move |_ctx, exp| match exp {
            Exp::Hole(Hole { metavar, .. }) => metavar.id == meta_var_id,
            _ => false,
        })
    }
}

pub fn occurs_in(ctx: &mut LevelCtx, the_idx: Idx, in_exp: &Exp) -> bool {
    let lvl = ctx.idx_to_lvl(the_idx);
    in_exp.occurs_var(ctx, lvl)
}

impl<T: Occurs> Occurs for Box<T> {
    fn occurs<F>(&self, ctx: &mut LevelCtx, f: &F) -> bool
    where
        F: Fn(&LevelCtx, &Exp) -> bool,
    {
        T::occurs(self, ctx, f)
    }
}

impl<T: Occurs> Occurs for Vec<T> {
    fn occurs<F>(&self, ctx: &mut LevelCtx, f: &F) -> bool
    where
        F: Fn(&LevelCtx, &Exp) -> bool,
    {
        self.iter().any(|x| x.occurs(ctx, f))
    }
}

impl<T: Occurs> Occurs for Option<T> {
    fn occurs<F>(&self, ctx: &mut LevelCtx, f: &F) -> bool
    where
        F: Fn(&LevelCtx, &Exp) -> bool,
    {
        self.as_ref().map(|inner| inner.occurs(ctx, f)).unwrap_or_default()
    }
}
