use codespan::Span;

mod de_bruijn;
mod equiv;
mod forget;
mod named;
mod subst;

pub use de_bruijn::*;
pub use equiv::*;
pub use forget::*;
pub use named::*;
pub use subst::*;

pub type Ident = String;

pub trait HasSpan {
    fn span(&self) -> Option<Span>;
}

pub trait HasInfo {
    type Info;

    fn info(&self) -> Self::Info;
}

#[derive(Debug, Clone)]
pub struct DocComment {
    pub docs: Vec<String>,
}

pub fn trim_newline(s: &mut String) -> String {
    if s.ends_with('\n') || s.ends_with('\r') {
        s.pop();
        if s.ends_with('\r') {
            s.pop();
        }
    };
    s.to_string()
}
