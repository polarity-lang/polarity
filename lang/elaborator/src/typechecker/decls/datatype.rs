//! Checking the well-formedness of toplevel data type declarations
use log::trace;
use miette_util::ToMiette;

use ast::*;

use crate::typechecker::{
    ctx::Ctx,
    exprs::{CheckInfer, InferTelescope},
    TypeError,
};

use super::CheckToplevel;

/// Check a data declaration
impl CheckToplevel for Data {
    fn check_wf(&self, ctx: &mut Ctx) -> Result<Self, TypeError> {
        trace!("Checking well-formedness of data type: {}", self.name);

        let Data { span, doc, name, attr, typ, ctors } = self;

        let typ_out = typ.infer_telescope(ctx, |_, params_out| Ok(params_out))?;

        let ctors =
            ctors.iter().map(|ctor| check_ctor_wf(name, ctor, ctx)).collect::<Result<_, _>>()?;

        Ok(Data {
            span: *span,
            doc: doc.clone(),
            name: name.clone(),
            attr: attr.clone(),
            typ: Box::new(typ_out),
            ctors,
        })
    }
}

/// Infer a constructor declaration
fn check_ctor_wf(data_type_name: &IdBind, ctor: &Ctor, ctx: &mut Ctx) -> Result<Ctor, TypeError> {
    trace!("Checking well-formedness of constructor: {}", ctor.name);

    let Ctor { span, doc, name, params, typ } = ctor;

    // Check that the constructor lies in the data type it is defined in
    if &typ.name != data_type_name {
        return Err(TypeError::NotInType {
            expected: Box::new(data_type_name.clone()),
            actual: Box::new(typ.name.clone()),
            span: typ.span.to_miette(),
        });
    }

    params.infer_telescope(ctx, |ctx, params_out| {
        let typ_out = typ.infer(ctx)?;

        Ok(Ctor {
            span: *span,
            doc: doc.clone(),
            name: name.clone(),
            params: params_out,
            typ: typ_out,
        })
    })
}
