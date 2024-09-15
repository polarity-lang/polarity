mod ctx;
mod lookup_table;
mod lower;
mod result;

use ast::{self};
use parser::cst;

use crate::lookup_table::build_lookup_table;
use crate::lower::Lower;

pub use ctx::*;
pub use result::*;

pub fn lower_module(prg: &cst::decls::Module) -> Result<ast::Module, LoweringError> {
    let cst::decls::Module { uri, use_decls, decls } = prg;

    let lookup_table = build_lookup_table(prg)?;

    let mut ctx = Ctx::empty(lookup_table);

    let use_decls = use_decls.lower(&mut ctx)?;
    let decls = decls.lower(&mut ctx)?;

    Ok(ast::Module { uri: uri.clone(), use_decls, decls, meta_vars: ctx.meta_vars })
}
