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

#[derive(Debug, Clone)]
pub enum BindingSite {
    Var { name: Ident },
    Wildcard,
}

pub trait HasSpan {
    fn span(&self) -> Option<Span>;
}

pub trait HasInfo {
    type Info;

    fn info(&self) -> Self::Info;
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
