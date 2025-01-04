//! High-level intermediate respresentation of the AST after erasure.
//! This representation is shared between any compiler backends and hence can only make few assumptions about the compilation target.

pub mod decls;
pub mod exprs;

pub use decls::*;
pub use exprs::*;
