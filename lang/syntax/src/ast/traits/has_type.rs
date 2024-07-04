use std::rc::Rc;

use crate::ast::exp::Exp;

/// Trait for expressions which have a type.
///
/// The function  `typ()` itself should not perform any non-trivial computation.
/// You should first run elaboration on an expression before you call `typ()` on it,
/// otherwise the function is not guaranteed to return a result.
pub trait HasType {
    /// Return the type of the expression.
    fn typ(&self) -> Option<Rc<Exp>>;
}
