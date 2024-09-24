use codespan::Span;
use derivative::Derivative;

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Ident {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub span: Span,
    pub id: String,
}
