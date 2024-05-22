use codespan::Span;

mod de_bruijn;

pub use de_bruijn::*;

pub type HashMap<K, V> = std::collections::HashMap<K, V, fxhash::FxBuildHasher>;
pub type HashSet<V> = fxhash::FxHashSet<V>;

pub trait HasSpan {
    fn span(&self) -> Option<Span>;
}

impl HasSpan for Option<Span> {
    fn span(&self) -> Option<Span> {
        *self
    }
}
