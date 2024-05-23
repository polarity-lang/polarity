use syntax::ast::*;

use crate::normalizer::{env::ToEnv, normalize::Normalize};

use crate::typechecker::{
    ctx::Ctx,
    typecheck::{CheckInfer, InferTelescope},
    TypeError,
};

use super::CheckToplevel;

impl CheckToplevel for Let {
    fn check_wf(&self, prg: &Module, ctx: &mut Ctx) -> Result<Self, TypeError> {
        let Let { span, doc, name, attr, params, typ, body } = self;

        params.infer_telescope(prg, ctx, |ctx, params_out| {
            let typ_out = typ.infer(prg, ctx)?;
            let typ_nf = typ.normalize(prg, &mut ctx.env())?;
            let body_out = body.check(prg, ctx, typ_nf)?;

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
