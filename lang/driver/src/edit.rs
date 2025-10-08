use std::ops::Range;

use polarity_lang_miette_util::codespan::Span;
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
            let remove_range = edit.span.as_range();
            let char_range =
                rope.byte_to_char(remove_range.start)..rope.byte_to_char(remove_range.end);
            rope.remove(char_range);
            let char_idx = rope.byte_to_char(edit.span.start.0 as usize);
            rope.insert(char_idx, &edit.text);
        }

        rope
    }
}

trait SpanAsRange {
    fn as_range(&self) -> Range<usize>;
}

impl SpanAsRange for Span {
    fn as_range(&self) -> Range<usize> {
        self.start.0 as usize..self.end.0 as usize
    }
}
