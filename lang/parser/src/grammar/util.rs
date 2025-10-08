use polarity_lang_miette_util::codespan::{ByteIndex, Span};

pub fn span(l: usize, r: usize) -> Span {
    Span { start: ByteIndex(l as u32), end: ByteIndex(r as u32) }
}
