mod decls;
mod exp;
mod ident;
mod lookup;
pub mod lookup_table;
pub mod traits;

pub use decls::*;
pub use exp::*;
pub use ident::*;
pub use lookup::*;
pub use traits::*;

pub type HashMap<K, V> = std::collections::HashMap<K, V, fxhash::FxBuildHasher>;
pub type HashSet<V> = fxhash::FxHashSet<V>;
