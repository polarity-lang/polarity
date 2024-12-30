use super::ctx::Ctx;
use super::result::ErasureError;
use super::symbol_table::{CodefMeta, DefMeta, LetMeta};
use super::traits::Erasure;

impl Erasure for ast::Module {
    fn erase(&mut self, ctx: &Ctx) -> Result<(), ErasureError> {
        let ast::Module { uri: _, use_decls: _, decls, meta_vars: _ } = self;

        erase_decls(ctx, decls)
    }
}

fn erase_decls(ctx: &Ctx, decls: &mut [ast::Decl]) -> Result<(), ErasureError> {
    for decl in decls {
        match decl {
            ast::Decl::Def(def) => def.erase(ctx)?,
            ast::Decl::Codef(codef) => codef.erase(ctx)?,
            ast::Decl::Let(tl_let) => tl_let.erase(ctx)?,
            ast::Decl::Data(_) => {}
            ast::Decl::Codata(_) => {}
        }
    }

    Ok(())
}

impl Erasure for ast::Def {
    fn erase(&mut self, ctx: &Ctx) -> Result<(), ErasureError> {
        let DefMeta { params: params_erased, erased, .. } =
            ctx.symbol_table.lookup_def(&ctx.uri, &self.name.id)?.clone();

        for (param, param_erased) in self.params.params.iter_mut().zip(params_erased.iter()) {
            param.erased = param_erased.erased;
        }

        self.erased = erased;

        for case in self.cases.iter_mut() {
            case.erase(ctx)?;
        }

        Ok(())
    }
}

impl Erasure for ast::Codef {
    fn erase(&mut self, ctx: &Ctx) -> Result<(), ErasureError> {
        let CodefMeta { params: params_erased, .. } =
            ctx.symbol_table.lookup_codef(&ctx.uri, &self.name.id)?.clone();

        for (param, param_erased) in self.params.params.iter_mut().zip(params_erased.iter()) {
            param.erased = param_erased.erased;
        }

        for case in self.cases.iter_mut() {
            case.erase(ctx)?;
        }

        Ok(())
    }
}

impl Erasure for ast::Let {
    fn erase(&mut self, ctx: &Ctx) -> Result<(), ErasureError> {
        let LetMeta { params: params_erased, erased, .. } =
            ctx.symbol_table.lookup_let(&ctx.uri, &self.name.id)?.clone();

        for (param, param_erased) in self.params.params.iter_mut().zip(params_erased.iter()) {
            param.erased = param_erased.erased;
        }

        self.erased = erased;

        self.body.erase(ctx)?;

        Ok(())
    }
}
