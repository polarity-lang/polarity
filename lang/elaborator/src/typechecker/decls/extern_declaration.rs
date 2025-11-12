//! Checking the well-formedness of extern declarations

use log::trace;

use polarity_lang_ast::*;

use super::CheckToplevel;
use crate::result::TcResult;
use crate::typechecker::erasure;
use crate::typechecker::{
    ctx::Ctx,
    exprs::{CheckInfer, InferTelescope},
};

impl CheckToplevel for Extern {
    fn check_wf(&self, ctx: &mut Ctx) -> TcResult<Self> {
        trace!("Checking well-formedness of extern declaration: {}", self.name);

        let Extern { span, doc, name, attr, params, typ } = self;

        params.infer_telescope(ctx, |ctx, mut params_out| {
            let typ_out = typ.check(ctx, &TypeUniv::new().into())?;

            erasure::mark_erased_params(&mut params_out);

            Ok(Extern {
                span: *span,
                doc: doc.clone(),
                name: name.clone(),
                attr: attr.clone(),
                params: params_out,
                typ: typ_out,
            })
        })
    }
}
