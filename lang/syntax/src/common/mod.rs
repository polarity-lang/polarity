mod de_bruijn;

pub use de_bruijn::*;

pub type HashMap<K, V> = std::collections::HashMap<K, V, fxhash::FxBuildHasher>;
pub type HashSet<V> = fxhash::FxHashSet<V>;
