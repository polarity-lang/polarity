//! Selection of data structures optimized for convenience in the compiler use case

mod dec;
mod hash_map;
mod hash_set;
pub mod string;

pub use dec::*;
pub use hash_map::*;
pub use hash_set::*;

pub use std::convert::identity as id;
