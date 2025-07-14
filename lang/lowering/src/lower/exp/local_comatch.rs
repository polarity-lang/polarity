use ast::FreeVars;
use parser::cst;

use crate::{Ctx, LoweringResult, lower::Lower};

use super::lower_telescope_inst;

impl Lower for cst::exp::LocalComatch {
    type Target = ast::Exp;

    fn lower(&self, ctx: &mut Ctx) -> LoweringResult<Self::Target> {
        let cst::exp::LocalComatch { span, name, is_lambda_sugar, cases } = self;
        let cases = cases.lower(ctx)?;
        let fvs = cases.free_vars(&ctx.binders);
        let closure = ast::Closure::identity(&ctx.binders, &fvs);
        Ok(ast::LocalComatch {
            span: Some(*span),
            ctx: None,
            name: ctx.unique_label(name.to_owned(), span)?,
            closure,
            is_lambda_sugar: *is_lambda_sugar,
            cases,
            inferred_type: None,
        }
        .into())
    }
}

impl Lower for cst::exp::Case<cst::exp::Copattern> {
    type Target = ast::Case;

    fn lower(&self, ctx: &mut Ctx) -> LoweringResult<Self::Target> {
        let cst::exp::Case { span, pattern, body } = self;

        lower_telescope_inst(&pattern.params, ctx, |ctx, params| {
            let (_, name) = ctx.symbol_table.lookup(&pattern.name)?;
            Ok(ast::Case {
                span: Some(*span),
                pattern: ast::Pattern {
                    span: Some(pattern.span),
                    is_copattern: true,
                    name,
                    params,
                },
                body: body.lower(ctx)?,
            })
        })
    }
}
