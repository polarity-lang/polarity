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
    fn whnf(&self, ctx: Closure) -> WHNFResult<MachineState<Self::Target>>;

    fn inline(&mut self, ctx: &Closure);

    /// Compute the weak head normal form of the expression in the given context
    /// and inline the resulting environment.
    /// For example, the WHNF of
    /// - `let x = 2 in Pair(x,x)`
    ///
    /// is the tuple
    /// - `Pair(2,2)`
    fn whnf_inline(&self, ctx: Closure) -> WHNFResult<Self::Target> {
        let (mut e, ctx, _) = self.whnf(ctx)?;
        e.inline(&ctx);
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

    fn inline(&mut self, ctx: &Closure) {
        if let Some(x) = self {
            x.inline(ctx)
        }
    }
}

impl<T: WHNF> WHNF for Box<T> {
    type Target = Box<T::Target>;

    fn whnf(&self, ctx: Closure) -> WHNFResult<MachineState<Self::Target>> {
        let (whnf, ctx, is_neutral) = (**self).whnf(ctx)?;
        Ok((Box::new(whnf), ctx, is_neutral))
    }

    fn inline(&mut self, ctx: &Closure) {
        (**self).inline(ctx);
    }
}
