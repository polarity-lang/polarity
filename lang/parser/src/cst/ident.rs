use derivative::Derivative;
use polarity_lang_miette_util::codespan::Span;

/// An unqualified identifier.
///
/// Example: `Vec`
#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Ident {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub span: Span,
    pub id: String,
}

/// A (possibly) qualified identifier.
///
/// Examples: `Vec`, `data::Vec`, `codata::Fun`
#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct QIdent {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub span: Span,
    pub quals: Vec<String>,
    pub id: String,
}

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Operator {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub span: Span,
    pub id: String,
}
