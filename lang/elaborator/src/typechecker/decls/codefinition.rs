//! Checking the well-formedness of codefinitions
use std::rc::Rc;

use log::trace;

use ast::*;

use crate::normalizer::{env::ToEnv, normalize::Normalize};

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

        params.infer_telescope(ctx, |ctx, params_out| {
            let typ_out = typ.check(ctx, Rc::new(TypeUniv::new().into()))?;
            let typ_nf = typ.normalize(&ctx.module, &mut ctx.env())?;
            let wd = WithExpectedType {
                cases,
                label: Some((name.to_owned(), params.len())),
                expected_type: typ_nf.expect_typ_app()?,
            };

            wd.check_exhaustiveness(&ctx.module)?;
            let cases = wd.infer_wd(ctx)?;

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
