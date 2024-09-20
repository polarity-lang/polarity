use miette_util::ToMiette;
use parser::cst::ident::Ident;
use parser::cst::{self};

use super::super::*;
use super::lower_self_param;
use super::lower_telescope;

impl Lower for cst::decls::Codata {
    type Target = ast::Codata;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        log::trace!("Lowering codata declaration: {}", self.name.id);
        let cst::decls::Codata { span, doc, name, attr, params, dtors } = self;

        let dtors = dtors
            .iter()
            .map(|dtor| lower_destructor(dtor, ctx, name, params.len()))
            .collect::<Result<_, _>>()?;

        Ok(ast::Codata {
            span: Some(*span),
            doc: doc.lower(ctx)?,
            name: name.id.clone(),
            attr: attr.lower(ctx)?,
            typ: Box::new(lower_telescope(params, ctx, |_, out| Ok(out))?),
            dtors,
        })
    }
}

fn lower_destructor(
    dtor: &cst::decls::Dtor,
    ctx: &mut Ctx,
    type_name: &Ident,
    type_arity: usize,
) -> Result<ast::Dtor, LoweringError> {
    log::trace!("Lowering destructor: {:?}", dtor.name);
    let cst::decls::Dtor { span, doc, name, params, destructee, ret_typ } = dtor;

    lower_telescope(params, ctx, |ctx, params| {
        // If the type constructor does not take any arguments, it can be left out
        let on_typ = match &destructee.typ {
            Some(on_typ) => on_typ.clone(),
            None => {
                if type_arity == 0 {
                    cst::exp::Call {
                        span: Default::default(),
                        name: type_name.clone(),
                        args: vec![],
                    }
                } else {
                    return Err(LoweringError::MustProvideArgs {
                        xtor: name.clone(),
                        typ: type_name.clone(),
                        span: span.to_miette(),
                    });
                }
            }
        };

        let self_param = cst::decls::SelfParam {
            span: destructee.span,
            name: destructee.name.clone(),
            typ: on_typ,
        };

        lower_self_param(&self_param, ctx, |ctx, self_param| {
            Ok(ast::Dtor {
                span: Some(*span),
                doc: doc.lower(ctx)?,
                name: name.id.clone(),
                params,
                self_param,
                ret_typ: ret_typ.lower(ctx)?,
            })
        })
    })
}
