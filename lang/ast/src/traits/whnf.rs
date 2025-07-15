use crate::Closure;

/// Expressions which can be evaluated to weak head normal form.
pub trait WHNF {
    type Target: WHNF;

    /// Compute the weak head normal form of the expression in the given context.
    /// For example, the WHNF of
    /// - `let x = 2 in Pair(x,x)`
    ///
    /// is the tuple
    /// - `(Pair(x,x), [x -> 2])`
    fn whnf(&self, ctx: Closure) -> (Self::Target, Closure);

    fn inline(&mut self, ctx: &Closure);

    /// Compute the weak head normal form of the expression in the given context
    /// and inline the resulting environment.
    /// For example, the WHNF of
    /// - `let x = 2 in Pair(x,x)`
    ///
    /// is the tuple
    /// - `Pair(2,2)`
    fn whnf_inline(&self, ctx: Closure) -> Self::Target {
        let (mut e, ctx) = self.whnf(ctx);
        e.inline(&ctx);
        e
    }
}
