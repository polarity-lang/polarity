//! Checking the well-formedness of global let-bound expressions

use log::trace;

use polarity_lang_ast::*;

use super::CheckToplevel;
use crate::normalizer::env::ToEnv;
use crate::normalizer::normalize::Normalize;
use crate::result::TcResult;
use crate::typechecker::erasure;
use crate::typechecker::{
    ctx::Ctx,
    exprs::{CheckInfer, InferTelescope},
};

impl CheckToplevel for Let {
    fn check_wf(&self, ctx: &mut Ctx) -> TcResult<Self> {
        trace!("Checking well-formedness of global let: {}", self.name);

        let Let { span, doc, name, attr, params, typ, body } = self;

        params.infer_telescope(ctx, |ctx, mut params_out| {
            let typ_out = typ.infer(ctx)?;
            let typ_nf = typ.normalize(&ctx.type_info_table, &mut ctx.env())?;
            let body_out = body.check(ctx, &typ_nf)?;

            erasure::mark_erased_params(&mut params_out);

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
