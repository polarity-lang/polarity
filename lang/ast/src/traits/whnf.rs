use crate::Closure;

/// Expressions into which we can inline a closure.
pub trait Inline {
    /// Replace every variable occurrence with the expression bound in `ctx`.
    /// The `recursive` argument determines what we do when we encounter a
    /// local match or comatch. If the `recursive` flag is true, then we
    /// recursively call `inline` on the right-hand sides of the (co)cases
    /// as well. Otherwise we only apply it in the locally bound closure.
    fn inline(&mut self, ctx: &Closure, recursive: bool);
}

impl<T: Inline> Inline for Option<T> {
    fn inline(&mut self, ctx: &Closure, recursive: bool) {
        if let Some(x) = self {
            x.inline(ctx, recursive)
        }
    }
}

impl<T: Inline> Inline for Box<T> {
    fn inline(&mut self, ctx: &Closure, recursive: bool) {
        (**self).inline(ctx, recursive);
    }
}

impl<T: Inline> Inline for Vec<T> {
    fn inline(&mut self, ctx: &Closure, recursive: bool) {
        for x in self {
            x.inline(ctx, recursive);
        }
    }
}

/// Expressions which can be evaluated to weak head normal form.
pub trait WHNF {
    type Target: Inline;

    /// Compute the weak head normal form of the expression in the given context.
    /// For example, the WHNF of
    /// - `let x = 2 in Pair(x,x)`
    ///
    /// is the tuple
    /// - `(Pair(x,x), [x -> 2])`
    fn whnf(&self, ctx: Closure) -> WHNFResult<MachineState<Self::Target>>;

    /// Compute the weak head normal form of the expression in the given context
    /// and inline the resulting environment.
    /// For example, the WHNF of
    /// - `let x = 2 in Pair(x,x)`
    ///
    /// is the tuple
    /// - `Pair(2,2)`
    fn whnf_inline(&self, ctx: Closure) -> WHNFResult<Self::Target> {
        let (mut e, ctx, _) = self.whnf(ctx)?;
        e.inline(&ctx, true);
        Ok(e)
    }
}

pub type WHNFResult<T> = Result<T, String>;

/// The machine state after one step of WHNF evaluation.
/// The elements have the following meaning:
/// - `T`: The WHNF expression.
/// - `Closure`: The new context.
/// - `is_neutral`: Whether the WHNF expression is neutral. If false, the expression is a value.
pub type MachineState<T> = (T, Closure, bool);

impl<T: WHNF> WHNF for Option<T> {
    type Target = Option<T::Target>;

    fn whnf(&self, ctx: Closure) -> WHNFResult<MachineState<Self::Target>> {
        match self {
            Some(x) => {
                let (whnf, ctx, is_neutral) = x.whnf(ctx)?;
                Ok((Some(whnf), ctx, is_neutral))
            }
            None => Ok((None, ctx, false)),
        }
    }
}

impl<T: WHNF> WHNF for Box<T> {
    type Target = Box<T::Target>;

    fn whnf(&self, ctx: Closure) -> WHNFResult<MachineState<Self::Target>> {
        let (whnf, ctx, is_neutral) = (**self).whnf(ctx)?;
        Ok((Box::new(whnf), ctx, is_neutral))
    }
}
