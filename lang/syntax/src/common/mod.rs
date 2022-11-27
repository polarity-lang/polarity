use codespan::Span;

mod de_bruijn;
mod equiv;
mod named;

pub use de_bruijn::*;
pub use equiv::*;
pub use named::*;

pub type Ident = String;

pub trait HasSpan {
    fn span(&self) -> Option<Span>;
}

pub trait HasInfo {
    type Info;

    fn info(&self) -> Self::Info;
}
