//! Type Erasure
//!
//! Mark terms as erased where applicable.
//! This is implemented as a transformation on the AST, making it easy to supply erasure information to the language server.
//! The actual transformation from AST to IR happens in the `backend` crate and relies on the erasure information in the AST generated here.

mod ctx;
mod decls;
mod exprs;
mod result;
mod symbol_table;
mod traits;

use ctx::Ctx;
pub use result::ErasureError;
pub use symbol_table::{
    build_erasure_symbol_table, GlobalErasureSymbolTable, ModuleErasureSymbolTable,
};
use traits::Erasure;

pub fn erase(
    symbol_table: GlobalErasureSymbolTable,
    module: &mut ast::Module,
) -> Result<(), ErasureError> {
    let ctx = Ctx::new(module.uri.clone(), symbol_table);
    module.erase(&ctx)
}
