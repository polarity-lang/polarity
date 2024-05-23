//! Checking the well-formedness of codefinitions
use std::rc::Rc;

use log::trace;

use syntax::ast::*;

use crate::normalizer::{env::ToEnv, normalize::Normalize};

use crate::typechecker::{
    ctx::Ctx,
    typecheck::{CheckInfer, InferTelescope, WithDestructee},
    util::ExpectTypApp,
    TypeError,
};

use super::CheckToplevel;

/// Infer a co-definition
impl CheckToplevel for Codef {
    fn check_wf(&self, prg: &Module, ctx: &mut Ctx) -> Result<Self, TypeError> {
        trace!("Checking well-formedness of codefinition: {}", self.name);

        let Codef { span, doc, name, attr, params, typ, body } = self;

        params.infer_telescope(prg, ctx, |ctx, params_out| {
            let typ_out = typ.check(prg, ctx, Rc::new(TypeUniv::new().into()))?;
            let typ_nf = typ.normalize(prg, &mut ctx.env())?;
            let wd = WithDestructee {
                inner: body,
                label: Some(name.to_owned()),
                n_label_args: params.len(),
                destructee: typ_nf.expect_typ_app()?,
            };
            let body_out = wd.infer_wd(prg, ctx)?;
            Ok(Codef {
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
