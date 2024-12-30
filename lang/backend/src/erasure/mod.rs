//! Type Erasure
//!
//! Convert AST to IR by erasing type information.
//!
//! Erasing types in a module proceeds in the following steps:
//!
//! 1. Build a `GlobalErasureSymbolTable` for the module and its dependencies using `build_erasure_symbol_table`
//! 2. Call `erase` on the module and its dependencies to generate IR
//! 3. Call `build_ir_symbol_table` to convert the erasure symbol tables to IR

mod ctx;
mod decls;
mod erasure_symbol_table;
mod exprs;
mod ir_symbol_table;
mod result;
mod traits;

use ctx::Ctx;
pub use erasure_symbol_table::{
    build_erasure_symbol_table, GlobalErasureSymbolTable, ModuleErasureSymbolTable,
};
pub use ir_symbol_table::build_ir_symbol_table;
pub use result::ErasureError;
use traits::Erasure;

pub fn erase(
    symbol_table: GlobalErasureSymbolTable,
    module: &ast::Module,
) -> Result<crate::ir::Module, ErasureError> {
    let mut ctx = Ctx::new(module.uri.clone(), symbol_table);
    module.erase(&mut ctx)
}
