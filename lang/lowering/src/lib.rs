mod ctx;
mod lower;
mod result;
mod symbol_table;

use polarity_lang_ast::{self};
use polarity_lang_miette_util::ToMiette;
use polarity_lang_parser::cst;
use polarity_lang_parser::cst::Ident;
use polarity_lang_parser::cst::ident::QIdent;

use crate::lower::Lower;

pub use ctx::*;
pub use result::*;
pub use symbol_table::DeclMeta;
pub use symbol_table::ModuleSymbolTable;
pub use symbol_table::SymbolTable;
pub use symbol_table::build::build_symbol_table;

/// This is a temporary function that we will remove once we have properly implemented qualified names in the lowering stage.
fn expect_ident(q: QIdent) -> LoweringResult<Ident> {
    if q.quals.is_empty() {
        Ok(Ident { span: q.span, id: q.id })
    } else {
        Err(Box::new(LoweringError::Impossible {
            message: "Qualified identifiers are not supported yet.".to_owned(),
            span: Some(q.span.to_miette()),
        }))
    }
}

/// Lower a module
///
/// The caller of this function needs to resolve module dependencies, lower all dependencies, and provide a symbol table with all symbols from these dependencies and the symbol table of the current module.
pub fn lower_module_with_symbol_table(
    prg: &cst::decls::Module,
    symbol_table: &SymbolTable,
) -> LoweringResult<polarity_lang_ast::Module> {
    let mut ctx = Ctx::empty(prg.uri.clone(), symbol_table.clone());

    let use_decls = prg.use_decls.lower(&mut ctx)?;
    let decls = prg.decls.lower(&mut ctx)?;

    Ok(polarity_lang_ast::Module {
        uri: prg.uri.clone(),
        use_decls,
        decls,
        meta_vars: ctx.meta_vars,
    })
}
