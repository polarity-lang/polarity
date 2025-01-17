//! Checking the well-formedness of codefinitions

use log::trace;

use ast::*;

use crate::normalizer::env::ToEnv;
use crate::normalizer::normalize::Normalize;

use crate::result::TcResult;
use crate::typechecker::erasure;
use crate::typechecker::exprs::local_comatch::WithExpectedType;
use crate::typechecker::{
    ctx::Ctx,
    exprs::{CheckInfer, InferTelescope},
    util::ExpectTypApp,
};

use super::CheckToplevel;

/// Infer a co-definition
impl CheckToplevel for Codef {
    fn check_wf(&self, ctx: &mut Ctx) -> TcResult<Self> {
        trace!("Checking well-formedness of codefinition: {}", self.name);

        let Codef { span, doc, name, attr, params, typ, cases } = self;

        let label = IdBound { span: name.span, id: name.id.clone(), uri: ctx.module.uri.clone() };

        params.infer_telescope(ctx, |ctx, mut params_out| {
            let typ_out = typ.check(ctx, &Box::new(TypeUniv::new().into()))?;
            let typ_nf = typ.normalize(&ctx.type_info_table, &mut ctx.env())?;
            let with_expected_type = WithExpectedType {
                cases,
                signature: Some((label, params)),
                expected_type: typ_nf.expect_typ_app()?,
            };

            with_expected_type.check_exhaustiveness(ctx)?;
            let cases = with_expected_type.check_type(ctx)?;

            erasure::mark_erased_params(&mut params_out);

            Ok(Codef {
                span: *span,
                doc: doc.clone(),
                name: name.clone(),
                attr: attr.clone(),
                params: params_out,
                typ: typ_out,
                cases,
            })
        })
    }
}
