use ast::IdBound;
use parser::cst;

use crate::{lower::Lower, Ctx, LoweringResult};

use super::lower_telescope_inst;

impl Lower for cst::exp::LocalMatch {
    type Target = ast::Exp;

    fn lower(&self, ctx: &mut Ctx) -> LoweringResult<Self::Target> {
        let cst::exp::LocalMatch { span, name, on_exp, motive, cases } = self;
        Ok(ast::LocalMatch {
            span: Some(*span),
            ctx: None,
            name: ctx.unique_label(name.to_owned(), span)?,
            on_exp: on_exp.lower(ctx)?,
            motive: motive.lower(ctx)?,
            ret_typ: None,
            cases: cases.lower(ctx)?,
            inferred_type: None,
        }
        .into())
    }
}

impl Lower for cst::exp::Case<cst::exp::Pattern> {
    type Target = ast::Case;

    fn lower(&self, ctx: &mut Ctx) -> LoweringResult<Self::Target> {
        let cst::exp::Case { span, pattern, body } = self;

        lower_telescope_inst(&pattern.params, ctx, |ctx, params| {
            let (_, uri) = ctx.symbol_table.lookup(&pattern.name)?;
            let name = IdBound {
                span: Some(pattern.name.span),
                id: pattern.name.id.clone(),
                uri: uri.clone(),
            };
            Ok(ast::Case {
                span: Some(*span),
                pattern: ast::Pattern {
                    span: Some(pattern.span),
                    is_copattern: false,
                    name,
                    params,
                },
                body: body.lower(ctx)?,
            })
        })
    }
}
