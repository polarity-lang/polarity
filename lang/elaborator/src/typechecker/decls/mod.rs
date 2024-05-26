mod codatatype;
mod codefinition;
mod datatype;
mod definition;
mod global_let;

use syntax::ast::*;

use super::{ctx::Ctx, TypeError};

pub fn check(prg: &Module) -> Result<Module, TypeError> {
    let mut ctx = Ctx::new(prg.meta_vars.clone());
    let mut prg = prg.check_wf(prg, &mut ctx)?;
    prg.meta_vars = ctx.meta_vars;
    Ok(prg)
}

pub trait CheckToplevel: Sized {
    fn check_wf(&self, prg: &Module, ctx: &mut Ctx) -> Result<Self, TypeError>;
}

/// Check all declarations in a program
impl CheckToplevel for Module {
    fn check_wf(&self, prg: &Module, ctx: &mut Ctx) -> Result<Self, TypeError> {
        let Module { uri, map, lookup_table, meta_vars: _ } = self;

        // FIXME: Reconsider order

        let map_out = map
            .iter()
            .map(|(name, decl)| Ok((name.clone(), decl.check_wf(prg, ctx)?)))
            .collect::<Result<_, TypeError>>()?;

        Ok(Module {
            uri: uri.clone(),
            map: map_out,
            lookup_table: lookup_table.clone(),
            meta_vars: self.meta_vars.clone(),
        })
    }
}

/// Check a declaration
impl CheckToplevel for Decl {
    fn check_wf(&self, prg: &Module, ctx: &mut Ctx) -> Result<Self, TypeError> {
        let out = match self {
            Decl::Data(data) => Decl::Data(data.check_wf(prg, ctx)?),
            Decl::Codata(codata) => Decl::Codata(codata.check_wf(prg, ctx)?),
            Decl::Ctor(ctor) => Decl::Ctor(ctor.check_wf(prg, ctx)?),
            Decl::Dtor(dtor) => Decl::Dtor(dtor.check_wf(prg, ctx)?),
            Decl::Def(def) => Decl::Def(def.check_wf(prg, ctx)?),
            Decl::Codef(codef) => Decl::Codef(codef.check_wf(prg, ctx)?),
            Decl::Let(tl_let) => Decl::Let(tl_let.check_wf(prg, ctx)?),
        };
        Ok(out)
    }
}
