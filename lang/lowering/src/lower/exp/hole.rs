use ast::Hole;
use parser::cst;

use crate::{lower::Lower, Ctx, LoweringResult};

impl Lower for cst::exp::HoleKind {
    type Target = ast::MetaVarKind;

    fn lower(&self, _ctx: &mut Ctx) -> LoweringResult<Self::Target> {
        match self {
            cst::exp::HoleKind::MustSolve => Ok(ast::MetaVarKind::MustSolve),
            cst::exp::HoleKind::CanSolve => Ok(ast::MetaVarKind::CanSolve),
        }
    }
}

impl Lower for cst::exp::Hole {
    type Target = ast::Exp;

    fn lower(&self, ctx: &mut Ctx) -> LoweringResult<Self::Target> {
        let cst::exp::Hole { span, kind, .. } = self;
        let kind = kind.lower(ctx)?;
        let mv = ctx.fresh_metavar(Some(*span), kind);
        let args = ctx.subst_from_ctx();
        Ok(Hole {
            span: Some(*span),
            kind,
            metavar: mv,
            inferred_type: None,
            inferred_ctx: None,
            args,
            solution: None,
        }
        .into())
    }
}
