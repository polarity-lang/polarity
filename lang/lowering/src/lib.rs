mod ctx;
mod lookup_table;
mod lower;
mod result;

use ast::{self, Decl};
use parser::cst;

use crate::lookup_table::build_lookup_table;
use crate::lower::Lower;

pub use ctx::*;
pub use result::*;

pub fn lower_module(prg: &cst::decls::Module) -> Result<ast::Module, LoweringError> {
    let cst::decls::Module { uri, contents } = prg;

    let lookup_table = build_lookup_table(prg)?;

    let mut ctx = Ctx::empty(lookup_table);
    let decls =
        contents.decls.iter().map(|item| item.lower(&mut ctx)).collect::<Result<Vec<Decl>, _>>()?;

    Ok(ast::Module { uri: uri.clone(), decls, meta_vars: ctx.meta_vars })
}
