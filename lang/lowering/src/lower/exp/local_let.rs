use miette_util::ToMiette;
use parser::cst;

use crate::{LoweringError, lower::Lower};

impl Lower for cst::exp::LocalLet {
    type Target = ast::Exp;

    fn lower(&self, _ctx: &mut crate::Ctx) -> crate::LoweringResult<Self::Target> {
        let cst::exp::LocalLet { span, .. } = self;
        Err(LoweringError::Impossible {
            message: "Lowering of local let expressions not implemented yet".to_string(),
            span: Some(span.to_miette()),
        }
        .into())
    }
}
