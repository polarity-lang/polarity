use crate::ir;

use super::ctx::Ctx;
use super::erasure_symbol_table::{CodefMeta, DefMeta, LetMeta};
use super::result::ErasureError;
use super::traits::Erasure;

impl Erasure for ast::Module {
    type Target = ir::Module;

    fn erase(&self, ctx: &mut Ctx) -> Result<Self::Target, ErasureError> {
        let ast::Module { uri, use_decls, decls, meta_vars: _ } = self;

        let ErasedDecls { def_decls, codef_decls, let_decls } = erase_decls(ctx, decls)?;

        Ok(ir::Module {
            uri: uri.clone(),
            use_decls: use_decls.clone(),
            def_decls,
            codef_decls,
            let_decls,
        })
    }
}

struct ErasedDecls {
    def_decls: Vec<ir::Def>,
    codef_decls: Vec<ir::Codef>,
    let_decls: Vec<ir::Let>,
}

fn erase_decls(ctx: &mut Ctx, decls: &[ast::Decl]) -> Result<ErasedDecls, ErasureError> {
    let mut def_decls = Vec::new();
    let mut codef_decls = Vec::new();
    let mut let_decls = Vec::new();

    for decl in decls {
        match decl {
            ast::Decl::Def(def) => {
                if let Some(def) = def.erase(ctx)? {
                    def_decls.push(def);
                }
            }
            ast::Decl::Codef(codef) => codef_decls.push(codef.erase(ctx)?),
            ast::Decl::Let(tl_let) => {
                if let Some(tl_let) = tl_let.erase(ctx)? {
                    let_decls.push(tl_let);
                }
            }
            ast::Decl::Data(_) => {}
            ast::Decl::Codata(_) => {}
        }
    }

    Ok(ErasedDecls { def_decls, codef_decls, let_decls })
}

impl Erasure for ast::Def {
    type Target = Option<ir::Def>;

    fn erase(&self, ctx: &mut Ctx) -> Result<Self::Target, ErasureError> {
        let ast::Def { name, cases, .. } = self;

        let DefMeta { name, self_param, params, erased } =
            ctx.symbol_table.lookup_def(&ctx.uri, &name.id)?.clone();

        if erased {
            return Ok(None);
        }

        let params_erased = params
            .iter()
            .filter(|param| !param.erased)
            .map(|param| param.name.to_string())
            .collect();

        let cases = ctx.bind(&params, |ctx| ctx.bind_single(false, |ctx| cases.erase(ctx)))?;

        Ok(Some(ir::Def { name, self_param, params: params_erased, cases }))
    }
}

impl Erasure for ast::Codef {
    type Target = ir::Codef;

    fn erase(&self, ctx: &mut Ctx) -> Result<Self::Target, ErasureError> {
        let ast::Codef { name, cases, .. } = self;

        let CodefMeta { name, params } = ctx.symbol_table.lookup_codef(&ctx.uri, &name.id)?.clone();

        let params_erased = params
            .iter()
            .filter(|param| !param.erased)
            .map(|param| param.name.to_string())
            .collect();

        let cases = ctx.bind(&params, |ctx| cases.erase(ctx))?;

        Ok(ir::Codef { name, params: params_erased, cases })
    }
}

impl Erasure for ast::Let {
    type Target = Option<ir::Let>;

    fn erase(&self, ctx: &mut Ctx) -> Result<Self::Target, ErasureError> {
        let ast::Let { name, body, .. } = self;

        let LetMeta { name, params, erased } =
            ctx.symbol_table.lookup_let(&ctx.uri, &name.id)?.clone();

        if erased {
            return Ok(None);
        }

        let params_erased = params
            .iter()
            .filter(|param| !param.erased)
            .map(|param| param.name.to_string())
            .collect();

        let body = ctx.bind(&params, |ctx| body.erase(ctx))?;
        let body = Box::new(body);

        Ok(Some(ir::Let { name, params: params_erased, body }))
    }
}
