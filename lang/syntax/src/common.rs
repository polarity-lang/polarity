use codespan::Span;

pub type Ident = String;

pub trait HasInfo {
    type Info;

    fn info(&self) -> &Self::Info;
    fn span(&self) -> Option<&Span>;
}
