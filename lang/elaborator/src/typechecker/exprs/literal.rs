use polarity_lang_ast::{HasSpan, Literal};

use crate::{
    conversion_checking::convert,
    result::TcResult,
    typechecker::{
        ctx::Ctx,
        exprs::{CheckInfer, ExpectType},
    },
};

impl CheckInfer for Literal {
    fn check(&self, ctx: &mut Ctx, t: &polarity_lang_ast::Exp) -> TcResult<Self> {
        let inferred_term = self.infer(ctx)?;
        let inferred_typ = inferred_term.expect_typ()?;
        convert(&ctx.vars, &mut ctx.meta_vars, inferred_typ, t, &self.span())?;
        Ok(inferred_term)
    }

    fn infer(&self, _ctx: &mut Ctx) -> TcResult<Self> {
        Ok(self.clone())
    }
}
