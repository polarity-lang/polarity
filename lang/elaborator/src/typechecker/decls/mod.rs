use std::rc::Rc;

mod codatatype;
mod codefinition;
mod datatype;
mod definition;
mod global_let;

use ast::*;

use super::{ctx::Ctx, type_info_table::TypeInfoTable, TypeError};

/// Check a module
///
/// The caller of this function needs to resolve module dependencies, check all dependencies, and provide a info table with all symbols from these dependencies.
pub fn check_with_lookup_table(
    prg: Rc<Module>,
    info_table: &TypeInfoTable,
) -> Result<Module, TypeError> {
    log::debug!("Checking module: {}", prg.uri);

    let mut ctx = Ctx::new(prg.meta_vars.clone(), info_table.clone(), prg.clone());

    let mut decls = prg
        .decls
        .iter()
        .map(|decl| decl.check_wf(&mut ctx))
        .collect::<Result<Vec<_>, TypeError>>()?;

    decls
        .zonk(&ctx.meta_vars)
        .map_err(|err| TypeError::Impossible { message: err.to_string(), span: None })?;

    ctx.check_metavars_solved()?;

    Ok(Module {
        uri: prg.uri.clone(),
        use_decls: prg.use_decls.clone(),
        decls,
        meta_vars: ctx.meta_vars.clone(),
    })
}

pub trait CheckToplevel: Sized {
    fn check_wf(&self, ctx: &mut Ctx) -> Result<Self, TypeError>;
}

/// Check a declaration
impl CheckToplevel for Decl {
    fn check_wf(&self, ctx: &mut Ctx) -> Result<Self, TypeError> {
        let out = match self {
            Decl::Data(data) => Decl::Data(data.check_wf(ctx)?),
            Decl::Codata(codata) => Decl::Codata(codata.check_wf(ctx)?),
            Decl::Def(def) => Decl::Def(def.check_wf(ctx)?),
            Decl::Codef(codef) => Decl::Codef(codef.check_wf(ctx)?),
            Decl::Let(tl_let) => Decl::Let(tl_let.check_wf(ctx)?),
        };
        Ok(out)
    }
}
