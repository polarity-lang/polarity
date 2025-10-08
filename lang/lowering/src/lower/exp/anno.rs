use polarity_lang_parser::cst;

use crate::{Ctx, LoweringResult, lower::Lower};

impl Lower for cst::exp::Anno {
    type Target = polarity_lang_ast::Exp;

    fn lower(&self, ctx: &mut Ctx) -> LoweringResult<Self::Target> {
        let cst::exp::Anno { span, exp, typ } = self;
        Ok(polarity_lang_ast::Anno {
            span: Some(*span),
            exp: exp.lower(ctx)?,
            typ: typ.lower(ctx)?,
            normalized_type: None,
        }
        .into())
    }
}
