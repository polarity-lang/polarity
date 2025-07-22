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
    /// is the expression
    /// - `Pair(2,2)`
    fn whnf(&self) -> WHNFResult<MachineState<Self::Target>>;
}

pub type WHNFResult<T> = Result<T, String>;

#[derive(PartialEq, Eq)]
pub enum IsWHNF {
    WHNF,
    Neutral,
}

/// The result of WHNF evaluation.
/// The elements have the following meaning:
/// - `T`: The WHNF expression.
/// - `is_neutral`: Whether the WHNF expression is neutral.
pub type MachineState<T> = (T, IsWHNF);

impl<T: WHNF> WHNF for Option<T> {
    type Target = Option<T::Target>;

    fn whnf(&self) -> WHNFResult<MachineState<Self::Target>> {
        match self {
            Some(x) => {
                let (whnf, is_whnf) = x.whnf()?;
                Ok((Some(whnf), is_whnf))
            }
            None => Ok((None, IsWHNF::Neutral)),
        }
    }
}

impl<T: WHNF> WHNF for Box<T> {
    type Target = Box<T::Target>;

    fn whnf(&self) -> WHNFResult<MachineState<Self::Target>> {
        let (whnf, is_whnf) = (**self).whnf()?;
        Ok((Box::new(whnf), is_whnf))
    }
}
