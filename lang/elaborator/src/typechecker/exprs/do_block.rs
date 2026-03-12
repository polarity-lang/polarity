use polarity_lang_ast::*;

use crate::result::TcResult;
use crate::typechecker::exprs::ExpectType;

use super::super::ctx::*;
use super::CheckInfer;

impl CheckInfer for DoBlock {
    fn check(&self, ctx: &mut Ctx, t: &polarity_lang_ast::Exp) -> TcResult<Self> {
        let DoBlock { span, statements, inferred_type: _ } = self;
        let statements = statements.check(ctx, t)?;
        let inferred_type = statements.expect_typ()?;
        Ok(DoBlock { span: *span, statements, inferred_type: Some(inferred_type) })
    }

    fn infer(&self, ctx: &mut crate::typechecker::ctx::Ctx) -> TcResult<Self> {
        let DoBlock { span, statements, inferred_type: _ } = self;
        let statements = statements.infer(ctx)?;
        let inferred_type = statements.expect_typ()?;
        Ok(DoBlock { span: *span, statements, inferred_type: Some(inferred_type) })
    }
}

impl CheckInfer for DoStatements {
    fn check(&self, ctx: &mut Ctx, t: &Exp) -> TcResult<Self> {
        todo!()
    }

    fn infer(&self, ctx: &mut Ctx) -> TcResult<Self> {
        todo!()
    }
}
