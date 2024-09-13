mod decls;
mod exp;
mod ident;
pub mod traits;

pub use decls::*;
pub use exp::*;
pub use ident::*;
pub use traits::*;

pub type HashMap<K, V> = std::collections::HashMap<K, V, fxhash::FxBuildHasher>;
pub type HashSet<V> = fxhash::FxHashSet<V>;
