use std::fmt;

use codespan::Span;

mod de_bruijn;
mod equiv;
mod forget;
mod named;
mod subst;

pub use de_bruijn::*;
use derivative::Derivative;
pub use equiv::*;
pub use forget::*;
pub use named::*;
pub use subst::*;

use parser::cst::Ident;

pub trait HasSpan {
    fn span(&self) -> Option<Span>;
}

impl HasSpan for Option<Span> {
    fn span(&self) -> Option<Span> {
        *self
    }
}

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Label {
    /// A machine-generated, unique id
    pub id: usize,
    /// A user-annotated name
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub user_name: Option<Ident>,
}

impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.user_name {
            None => Ok(()),
            Some(user_name) => user_name.fmt(f),
        }
    }
}
