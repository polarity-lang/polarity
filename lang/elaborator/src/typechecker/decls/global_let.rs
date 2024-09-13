//! Checking the well-formedness of global let-bound expressions

use log::trace;

use syntax::ast::*;

use super::CheckToplevel;
use crate::normalizer::{env::ToEnv, normalize::Normalize};
use crate::typechecker::{
    ctx::Ctx,
    exprs::{CheckInfer, InferTelescope},
    TypeError,
};

impl CheckToplevel for Let {
    fn check_wf(&self, ctx: &mut Ctx) -> Result<Self, TypeError> {
        trace!("Checking well-formedness of global let: {}", self.name);

        let Let { span, doc, name, attr, params, typ, body } = self;

        params.infer_telescope(ctx, |ctx, params_out| {
            let typ_out = typ.infer(ctx)?;
            let typ_nf = typ.normalize(&ctx.module, &mut ctx.env())?;
            let body_out = body.check(ctx, typ_nf)?;

            Ok(Let {
                span: *span,
                doc: doc.clone(),
                name: name.clone(),
                attr: attr.clone(),
                params: params_out,
                typ: typ_out,
                body: body_out,
            })
        })
    }
}
