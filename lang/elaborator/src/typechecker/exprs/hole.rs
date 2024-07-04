//! Bidirectional type checker

use std::rc::Rc;

use miette_util::ToMiette;
use syntax::ast::*;

use super::super::ctx::*;
use super::CheckInfer;
use crate::result::TypeError;

// Hole
//
//

impl CheckInfer for Hole {
    fn check(&self, prg: &Module, ctx: &mut Ctx, t: Rc<Exp>) -> Result<Self, TypeError> {
        let Hole { span, metavar, args, .. } = self;
        let args: Vec<Vec<Rc<Exp>>> = args
            .iter()
            .map(|subst| subst.iter().map(|exp| exp.infer(prg, ctx)).collect::<Result<Vec<_>, _>>())
            .collect::<Result<_, _>>()?;
        Ok(Hole {
            span: *span,
            metavar: *metavar,
            inferred_type: Some(t.clone()),
            inferred_ctx: Some(ctx.vars.clone()),
            args,
        })
    }

    fn infer(&self, _prg: &Module, _ctx: &mut Ctx) -> Result<Self, TypeError> {
        Err(TypeError::CannotInferHole { span: self.span().to_miette() })
    }
}
