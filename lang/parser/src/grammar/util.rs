use miette_util::codespan::{ByteIndex, Span};

pub fn span(l: usize, r: usize) -> Span {
    Span::new(ByteIndex(l as u32), ByteIndex(r as u32))
}
