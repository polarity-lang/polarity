//! Checking the well-formedness of toplevel codata type declarations
use log::trace;
use miette_util::ToMiette;

use ast::*;

use crate::{
    result::TcResult,
    typechecker::{
        TypeError,
        ctx::Ctx,
        erasure,
        exprs::{CheckInfer, InferTelescope},
    },
};

use super::CheckToplevel;

/// Infer a codata declaration
impl CheckToplevel for Codata {
    fn check_wf(&self, ctx: &mut Ctx) -> TcResult<Self> {
        trace!("Checking well-formedness of codata type: {}", self.name);

        let Codata { span, doc, name, attr, typ, dtors } = self;

        let typ_out = typ.infer_telescope(ctx, |_, params_out| Ok(params_out))?;

        let dtors =
            dtors.iter().map(|dtor| check_dtor_wf(name, dtor, ctx)).collect::<Result<_, _>>()?;

        Ok(Codata {
            span: *span,
            doc: doc.clone(),
            name: name.clone(),
            attr: attr.clone(),
            typ: Box::new(typ_out),
            dtors,
        })
    }
}

/// Infer a destructor declaration
fn check_dtor_wf(codata_name: &IdBind, dtor: &Dtor, ctx: &mut Ctx) -> TcResult<Dtor> {
    trace!("Checking well-formedness of destructor: {}", dtor.name);

    let Dtor { span, doc, name, params, self_param, ret_typ } = dtor;

    // Check that the destructor lies in the codata type it is defined in
    if &self_param.typ.name != codata_name {
        return Err(TypeError::NotInType {
            expected: Box::new(codata_name.clone()),
            actual: Box::new(self_param.typ.name.clone()),
            span: self_param.typ.span.to_miette(),
        }
        .into());
    }

    params.infer_telescope(ctx, |ctx, mut params_out| {
        self_param.infer_telescope(ctx, |ctx, self_param_out| {
            let ret_typ_out = ret_typ.infer(ctx)?;

            erasure::mark_erased_params(&mut params_out);

            Ok(Dtor {
                span: *span,
                doc: doc.clone(),
                name: name.clone(),
                params: params_out,
                self_param: self_param_out,
                ret_typ: ret_typ_out,
            })
        })
    })
}
