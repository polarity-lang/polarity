//! High-level intermediate respresentation of the AST after erasure.
//! This representation is shared between any compiler backends and hence can only make few assumptions about the compilation target.

pub mod decls;
pub mod exprs;
pub mod ident;
pub mod rename;

pub use decls::*;
pub use exprs::*;
pub use ident::Ident;
