//! Checking the well-formedness of codefinitions

use log::trace;

use ast::*;

use crate::normalizer::env::ToEnv;
use crate::normalizer::normalize::Normalize;

use crate::typechecker::exprs::local_comatch::WithExpectedType;
use crate::typechecker::{
    ctx::Ctx,
    exprs::{CheckInfer, InferTelescope},
    util::ExpectTypApp,
    TypeError,
};

use super::CheckToplevel;

/// Infer a co-definition
impl CheckToplevel for Codef {
    fn check_wf(&self, ctx: &mut Ctx) -> Result<Self, TypeError> {
        trace!("Checking well-formedness of codefinition: {}", self.name);

        let Codef { span, doc, name, attr, params, typ, cases } = self;

        let label = IdBound { span: name.span, id: name.id.clone(), uri: ctx.module.uri.clone() };

        params.infer_telescope(ctx, |ctx, params_out| {
            let typ_out = typ.check(ctx, &Box::new(TypeUniv::new().into()))?;
            let typ_nf = typ.normalize(&ctx.type_info_table, &mut ctx.env())?;
            let wd = WithExpectedType {
                cases,
                label: Some((label, params.len())),
                expected_type: typ_nf.expect_typ_app()?,
            };

            wd.check_exhaustiveness(ctx)?;
            let cases = wd.infer_wd(ctx, span)?;

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
