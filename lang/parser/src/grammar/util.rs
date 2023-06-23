use codespan::Span;

pub fn span(l: usize, r: usize) -> Span {
    Span::new(l as u32, r as u32)
}
