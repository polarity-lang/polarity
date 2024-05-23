use std::rc::Rc;

use miette_util::ToMiette;
use syntax::ast::*;


use crate::typechecker::{
    ctx::Ctx,
    typecheck::{CheckInfer, InferTelescope},
    TypeError,
};

use super::CheckToplevel;

/// Check a data declaration
impl CheckToplevel for Data {
    fn check_wf(&self, prg: &Module, ctx: &mut Ctx) -> Result<Self, TypeError> {
        let Data { span, doc, name, attr, typ, ctors } = self;

        let typ_out = typ.infer_telescope(prg, ctx, |_, params_out| Ok(params_out))?;

        Ok(Data {
            span: *span,
            doc: doc.clone(),
            name: name.clone(),
            attr: attr.clone(),
            typ: Rc::new(typ_out),
            ctors: ctors.clone(),
        })
    }
}

/// Infer a constructor declaration
impl CheckToplevel for Ctor {
    fn check_wf(&self, prg: &Module, ctx: &mut Ctx) -> Result<Self, TypeError> {
        let Ctor { span, doc, name, params, typ } = self;

        // Check that the constructor lies in the data type it is defined in
        let data_type = prg.data_for_ctor(name, *span)?;
        let expected = &data_type.name;
        if &typ.name != expected {
            return Err(TypeError::NotInType {
                expected: expected.clone(),
                actual: typ.name.clone(),
                span: typ.span.to_miette(),
            });
        }

        params.infer_telescope(prg, ctx, |ctx, params_out| {
            let typ_out = typ.infer(prg, ctx)?;

            Ok(Ctor {
                span: *span,
                doc: doc.clone(),
                name: name.clone(),
                params: params_out,
                typ: typ_out,
            })
        })
    }
}
