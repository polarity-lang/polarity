//! Checking the well-formedness of toplevel codata type declarations
use std::rc::Rc;

use log::trace;
use miette_util::ToMiette;

use syntax::ast::*;

use crate::typechecker::{
    ctx::Ctx,
    typecheck::{CheckInfer, InferTelescope},
    TypeError,
};

use super::CheckToplevel;

/// Infer a codata declaration
impl CheckToplevel for Codata {
    fn check_wf(&self, prg: &Module, ctx: &mut Ctx) -> Result<Self, TypeError> {
        trace!("Checking well-formedness of codata type: {}", self.name);

        let Codata { span, doc, name, attr, typ, dtors } = self;

        let typ_out = typ.infer_telescope(prg, ctx, |_, params_out| Ok(params_out))?;

        Ok(Codata {
            span: *span,
            doc: doc.clone(),
            name: name.clone(),
            attr: attr.clone(),
            typ: Rc::new(typ_out),
            dtors: dtors.clone(),
        })
    }
}

/// Infer a destructor declaration
impl CheckToplevel for Dtor {
    fn check_wf(&self, prg: &Module, ctx: &mut Ctx) -> Result<Self, TypeError> {
        trace!("Checking well-formedness of destructor: {}", self.name);

        let Dtor { span, doc, name, params, self_param, ret_typ } = self;

        // Check that the destructor lies in the codata type it is defined in
        let codata_type = prg.codata_for_dtor(name, *span)?;
        let expected = &codata_type.name;
        if &self_param.typ.name != expected {
            return Err(TypeError::NotInType {
                expected: expected.clone(),
                actual: self_param.typ.name.clone(),
                span: self_param.typ.span.to_miette(),
            });
        }

        params.infer_telescope(prg, ctx, |ctx, params_out| {
            self_param.infer_telescope(prg, ctx, |ctx, self_param_out| {
                let ret_typ_out = ret_typ.infer(prg, ctx)?;

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
}
