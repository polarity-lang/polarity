use crate::Closure;


/// Expressions which can be evaluated to weak head normal form.
pub trait WHNF {
    type Target;

    /// Compute the weak head normal form of the expression in the given context.
    fn whnf(&self, ctx: Closure) -> Self::Target;
}