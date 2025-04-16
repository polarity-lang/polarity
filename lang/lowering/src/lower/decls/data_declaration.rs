use ast::{IdBind, IdBound};
use miette_util::ToMiette;
use parser::cst::ident::Ident;
use parser::cst::{self};

use super::super::*;
use super::lower_telescope;

impl Lower for cst::decls::Data {
    type Target = ast::Data;

    fn lower(&self, ctx: &mut Ctx) -> LoweringResult<Self::Target> {
        log::trace!("Lowering data declaration: {}", self.name.id);
        let cst::decls::Data { span, doc, name, attr, params, ctors } = self;

        let ctors = ctors
            .iter()
            .map(|ctor| lower_constructor(ctor, ctx, name, params.len()))
            .collect::<Result<_, _>>()?;

        Ok(ast::Data {
            span: Some(*span),
            doc: doc.lower(ctx)?,
            name: IdBind { span: Some(name.span), id: name.id.clone() },
            attr: attr.lower(ctx)?,
            typ: Box::new(lower_telescope(params, ctx, |_, out| Ok(out))?),
            ctors,
        })
    }
}

fn lower_constructor(
    ctor: &cst::decls::Ctor,
    ctx: &mut Ctx,
    typ_name: &Ident,
    type_arity: usize,
) -> LoweringResult<ast::Ctor> {
    log::trace!("Lowering constructor: {:?}", ctor.name);
    let cst::decls::Ctor { span, doc, name, params, typ } = ctor;

    lower_telescope(params, ctx, |ctx, params| {
        // If the type constructor does not take any arguments, it can be left out
        let typ = match typ {
            Some(typ) => typ
                .lower(ctx)?
                .to_typctor()
                .ok_or(LoweringError::ExpectedTypCtor { span: span.to_miette() })?,
            None => {
                if type_arity == 0 {
                    ast::TypCtor {
                        span: None,
                        name: IdBound {
                            span: Some(typ_name.span),
                            id: typ_name.id.clone(),
                            uri: ctx.uri.clone(),
                        },
                        args: ast::Args { args: vec![] },
                        is_bin_op: None,
                    }
                } else {
                    return Err(LoweringError::MustProvideArgs {
                        xtor: name.clone(),
                        typ: typ_name.clone(),
                        span: span.to_miette(),
                    }
                    .into());
                }
            }
        };

        Ok(ast::Ctor {
            span: Some(*span),
            doc: doc.lower(ctx)?,
            name: IdBind { span: Some(name.span), id: name.id.clone() },
            params,
            typ,
        })
    })
}
