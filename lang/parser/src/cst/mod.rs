//! # Concrete syntax tree (CST)
//!
//! This representation is used as the output of the parser and as the input of the lowering stage that follows
//! in the compiler pipeline. The structure of the CST therefore corresponds closely to the grammar of the surface
//! syntax as implemented by the parser.

pub mod decls;
pub mod exp;
pub mod ident;

pub use ident::Ident;
