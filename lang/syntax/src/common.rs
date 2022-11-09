use codespan::Span;

pub type Ident = String;

pub trait HasSpan {
    fn span(&self) -> Option<Span>;
}

pub trait HasInfo {
    type Info;

    fn info(&self) -> Self::Info;
}
