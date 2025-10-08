use polarity_lang_parser::cst;

use crate::{Ctx, LoweringResult, lower::Lower};

impl Lower for cst::exp::Lam {
    type Target = polarity_lang_ast::Exp;

    fn lower(&self, ctx: &mut Ctx) -> LoweringResult<Self::Target> {
        let cst::exp::Lam { span, case } = self;
        let comatch = cst::exp::Exp::LocalComatch(cst::exp::LocalComatch {
            span: *span,
            name: None,
            is_lambda_sugar: true,
            cases: vec![case.clone()],
        });
        comatch.lower(ctx)
    }
}
