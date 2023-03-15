use std::fmt;

use codespan::Span;

mod de_bruijn;
mod equiv;
mod forget;
mod named;
mod subst;

pub use de_bruijn::*;
pub use equiv::*;
pub use forget::*;
pub use named::*;
pub use subst::*;

pub type Ident = String;

pub trait HasSpan {
    fn span(&self) -> Option<Span>;
}

pub trait HasInfo {
    type Info;

    fn info(&self) -> Self::Info;
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

#[derive(Debug, Clone)]
pub struct DocComment {
    pub docs: Vec<String>,
}

#[derive(Debug, Copy, Clone)]
pub enum HoleKind {
    Todo,
    Omitted,
}
