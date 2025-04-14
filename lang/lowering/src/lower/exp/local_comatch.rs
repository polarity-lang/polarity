use parser::cst;

use crate::{lower::Lower, Ctx, LoweringResult};

use super::lower_telescope_inst;

impl Lower for cst::exp::LocalComatch {
    type Target = ast::Exp;

    fn lower(&self, ctx: &mut Ctx) -> LoweringResult<Self::Target> {
        let cst::exp::LocalComatch { span, name, is_lambda_sugar, cases } = self;
        Ok(ast::LocalComatch {
            span: Some(*span),
            ctx: None,
            name: ctx.unique_label(name.to_owned(), span)?,
            is_lambda_sugar: *is_lambda_sugar,
            cases: cases.lower(ctx)?,
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
            let (_, uri) = ctx.symbol_table.lookup(&pattern.name)?;
            let name = ast::IdBound {
                span: Some(pattern.name.span),
                id: pattern.name.id.clone(),
                uri: uri.clone(),
            };
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
