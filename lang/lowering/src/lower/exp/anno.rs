use parser::cst;

use crate::{lower::Lower, Ctx, LoweringResult};

impl Lower for cst::exp::Anno {
    type Target = ast::Exp;

    fn lower(&self, ctx: &mut Ctx) -> LoweringResult<Self::Target> {
        let cst::exp::Anno { span, exp, typ } = self;
        Ok(ast::Anno {
            span: Some(*span),
            exp: exp.lower(ctx)?,
            typ: typ.lower(ctx)?,
            normalized_type: None,
        }
        .into())
    }
}
