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

pub fn lower_module(prg: &cst::decls::Module) -> Result<ast::Module, LoweringError> {
    lower_module_with_lookup_table(prg, &mut LookupTable::default())
}

pub fn lower_module_with_lookup_table(
    prg: &cst::decls::Module,
    lookup_table: &mut LookupTable,
) -> Result<ast::Module, LoweringError> {
    let mut combined_table = std::mem::take(lookup_table);
    combined_table.append(build_lookup_table(prg)?);

    let mut ctx = Ctx::empty(combined_table);

    let use_decls = prg.use_decls.lower(&mut ctx)?;
    let decls = prg.decls.lower(&mut ctx)?;

    *lookup_table = ctx.lookup_table;

    Ok(ast::Module { uri: prg.uri.clone(), use_decls, decls, meta_vars: ctx.meta_vars })
}
