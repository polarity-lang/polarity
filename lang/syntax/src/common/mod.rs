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

use crate::cst::Ident;

pub trait HasSpan {
    fn span(&self) -> Option<Span>;
}

pub trait HasInfo {
    type Info;

    fn info(&self) -> Self::Info;
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

#[derive(Clone, Copy, Debug)]
pub enum DeclKind {
    Data,
    Codata,
    Def,
    Codef,
    Ctor,
    Dtor,
}

impl fmt::Display for DeclKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeclKind::Data => write!(f, "data type"),
            DeclKind::Codata => write!(f, "codata type"),
            DeclKind::Def => write!(f, "definition"),
            DeclKind::Codef => write!(f, "codefinition"),
            DeclKind::Ctor => write!(f, "constructor"),
            DeclKind::Dtor => write!(f, "destructor"),
        }
    }
}
