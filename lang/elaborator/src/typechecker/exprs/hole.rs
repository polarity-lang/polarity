//! Bidirectional type checker

use ast::*;
use miette_util::ToMiette;

use super::super::ctx::*;
use super::CheckInfer;
use crate::result::TypeError;

// Hole
//
//

impl CheckInfer for Hole {
    fn check(&self, ctx: &mut Ctx, t: &Exp) -> Result<Self, TypeError> {
        let Hole { span, kind, metavar, args, solution, .. } = self;
        let args: Vec<Vec<Box<Exp>>> = args
            .iter()
            .map(|subst| subst.iter().map(|exp| exp.infer(ctx)).collect::<Result<Vec<_>, _>>())
            .collect::<Result<_, _>>()?;
        Ok(Hole {
            span: *span,
            kind: *kind,
            metavar: *metavar,
            inferred_type: Some(Box::new(t.clone())),
            inferred_ctx: Some(ctx.vars.clone()),
            args,
            solution: solution.clone(),
        })
    }

    fn infer(&self, __ctx: &mut Ctx) -> Result<Self, TypeError> {
        Err(TypeError::CannotInferHole { span: self.span().to_miette() })
    }
}
