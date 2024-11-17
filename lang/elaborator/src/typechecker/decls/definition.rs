//! Checking the well-formedness of definitions
use log::trace;

use ast::*;

use super::CheckToplevel;
use crate::normalizer::env::ToEnv;
use crate::normalizer::normalize::Normalize;
use crate::typechecker::exprs::local_match::WithScrutinee;
use crate::typechecker::{
    ctx::Ctx,
    exprs::{CheckInfer, InferTelescope},
    util::ExpectTypApp,
    TypeError,
};

/// Infer a definition
impl CheckToplevel for Def {
    fn check_wf(&self, ctx: &mut Ctx) -> Result<Self, TypeError> {
        trace!("Checking well-formedness of definition: {}", self.name);

        let Def { span, doc, name, attr, params, self_param, ret_typ, cases } = self;

        params.infer_telescope(ctx, |ctx, params_out| {
            let self_param_nf = self_param.typ.normalize(&ctx.type_info_table, &mut ctx.env())?;

            let (ret_typ_out, ret_typ_nf, self_param_out) =
                self_param.infer_telescope(ctx, |ctx, self_param_out| {
                    let ret_typ_out = ret_typ.infer(ctx)?;
                    let ret_typ_nf = ret_typ.normalize(&ctx.type_info_table, &mut ctx.env())?;
                    Ok((ret_typ_out, ret_typ_nf, self_param_out))
                })?;

            let ws = WithScrutinee { cases, scrutinee: self_param_nf.expect_typ_app()? };
            ws.check_exhaustiveness(ctx)?;
            let cases = ws.check_ws(ctx, &ret_typ_nf)?;
            Ok(Def {
                span: *span,
                doc: doc.clone(),
                name: name.clone(),
                attr: attr.clone(),
                params: params_out,
                self_param: self_param_out,
                ret_typ: ret_typ_out,
                cases,
            })
        })
    }
}
