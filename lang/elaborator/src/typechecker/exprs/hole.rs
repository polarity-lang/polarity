//! Bidirectional type checker

use ast::{ctx::values::Binder, *};
use miette_util::ToMiette;

use super::super::ctx::*;
use super::CheckInfer;
use crate::result::{TcResult, TypeError};

// Hole
//
//

impl CheckInfer for Hole {
    fn check(&self, ctx: &mut Ctx, t: &Exp) -> TcResult<Self> {
        let Hole { span, kind, metavar, args, solution, .. } = self;
        let args: Vec<Vec<Binder<Box<Exp>>>> = args
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

    fn infer(&self, __ctx: &mut Ctx) -> TcResult<Self> {
        Err(TypeError::CannotInferHole { span: self.span().to_miette() }.into())
    }
}

impl<T: CheckInfer> CheckInfer for Binder<T> {
    fn check(&self, ctx: &mut Ctx, t: &Exp) -> TcResult<Self> {
        let Binder { name, content } = self;

        Ok(Binder { name: name.clone(), content: content.check(ctx, t)? })
    }

    fn infer(&self, ctx: &mut Ctx) -> TcResult<Self> {
        let Binder { name, content } = self;

        Ok(Binder { name: name.clone(), content: content.infer(ctx)? })
    }
}
