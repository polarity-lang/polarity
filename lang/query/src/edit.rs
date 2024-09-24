use std::ops::Range;

use codespan::Span;
use ropey::Rope;
use url::Url;

use crate::database::Database;

#[derive(Ord, PartialOrd, Eq, PartialEq)]
pub struct Edit {
    pub span: Span,
    pub text: String,
}

impl Database {
    pub fn edited(&self, uri: &Url, mut edits: Vec<Edit>) -> Rope {
        let source = &self.files.get_even_if_stale(uri).unwrap().source;

        let mut rope = Rope::from_str(source);

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
