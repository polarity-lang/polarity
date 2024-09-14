use std::ops::Range;

use codespan::Span;
use ropey::Rope;

use crate::DatabaseViewMut;

#[derive(Ord, PartialOrd, Eq, PartialEq)]
pub struct Edit {
    pub span: Span,
    pub text: String,
}

impl<'a> DatabaseViewMut<'a> {
    pub fn edited(&self, mut edits: Vec<Edit>) -> Rope {
        let mut rope = Rope::from_str(self.source());

        edits.sort();

        for edit in edits.iter().rev() {
            rope.remove(edit.span.as_range());
            rope.insert(edit.span.start().into(), &edit.text);
        }

        rope
    }
}

trait SpanAsRange {
    fn as_range(&self) -> Range<usize>;
}

impl SpanAsRange for Span {
    fn as_range(&self) -> Range<usize> {
        self.start().into()..self.end().into()
    }
}
