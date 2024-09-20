use std::rc::Rc;

mod codatatype;
mod codefinition;
mod datatype;
mod definition;
mod global_let;

use ast::*;

use super::{
    ctx::Ctx,
    lookup_table::{build_lookup_table, LookupTable},
    TypeError,
};

pub fn check_with_lookup_table(
    prg: Rc<Module>,
    lookup_table: &mut LookupTable,
) -> Result<Module, TypeError> {
    log::debug!("Checking module: {}", prg.uri);

    let mut combined_table = std::mem::take(lookup_table);
    combined_table.append(build_lookup_table(&prg));
    let mut ctx = Ctx::new(prg.meta_vars.clone(), combined_table, prg.clone());

    let decls =
        prg.decls.iter().map(|decl| decl.check_wf(&mut ctx)).collect::<Result<_, TypeError>>()?;

    ctx.check_metavars_solved()?;

    *lookup_table = Rc::unwrap_or_clone(ctx.lookup_table);

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
