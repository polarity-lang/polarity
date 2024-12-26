use miette_util::codespan::Span;

/// Trait for syntactic entities which have a source-code span.
///
/// The function `span()` should return  `Some(span)` for every entity which
/// is the result of parsing or lowering, but might return `None` for
/// expressions which were annotated during elaboration, or which are the
/// result of some code transformation.
pub trait HasSpan {
    /// Return the source code span of the entity.
    fn span(&self) -> Option<Span>;
}

impl HasSpan for Option<Span> {
    fn span(&self) -> Option<Span> {
        *self
    }
}
