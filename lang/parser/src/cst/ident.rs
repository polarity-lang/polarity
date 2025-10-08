use derivative::Derivative;
use polarity_lang_miette_util::codespan::Span;

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Ident {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub span: Span,
    pub id: String,
}

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Operator {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub span: Span,
    pub id: String,
}
