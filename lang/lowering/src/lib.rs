mod ctx;
mod lookup_table;
mod lower;
mod result;

use ast::{self};
use parser::cst;

use crate::lower::Lower;

pub use ctx::*;
pub use lookup_table::{build_lookup_table, LookupTable};
pub use result::*;

/// Lower a module
///
/// The caller of this function needs to resolve module dependencies, lower all dependencies, and provide a lookup table with all symbols from these dependencies and the lookup table of the current module.
pub fn lower_module_with_lookup_table(
    prg: &cst::decls::Module,
    lookup_table: &LookupTable,
) -> Result<ast::Module, LoweringError> {
    let mut ctx = Ctx::empty(lookup_table.clone());

    let use_decls = prg.use_decls.lower(&mut ctx)?;
    let decls = prg.decls.lower(&mut ctx)?;

    Ok(ast::Module { uri: prg.uri.clone(), use_decls, decls, meta_vars: ctx.meta_vars })
}
