mod ctx;
mod lower;
mod result;
mod symbol_table;

use ast::{self};
use parser::cst;

use crate::lower::Lower;

pub use ctx::*;
pub use result::*;
pub use symbol_table::build::build_symbol_table;
pub use symbol_table::SymbolTable;

/// Lower a module
///
/// The caller of this function needs to resolve module dependencies, lower all dependencies, and provide a symbol table with all symbols from these dependencies and the symbol table of the current module.
pub fn lower_module_with_symbol_table(
    prg: &cst::decls::Module,
    symbol_table: &SymbolTable,
) -> Result<ast::Module, LoweringError> {
    let mut ctx = Ctx::empty(symbol_table.clone());

    let use_decls = prg.use_decls.lower(&mut ctx)?;
    let decls = prg.decls.lower(&mut ctx)?;

    Ok(ast::Module { uri: prg.uri.clone(), use_decls, decls, meta_vars: ctx.meta_vars })
}
