use miette_util::ToMiette;
use parser::cst::ident::Ident;
use parser::cst::{self};

use super::super::*;
use super::lower_telescope;

impl Lower for cst::decls::Data {
    type Target = ast::Data;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        log::trace!("Lowering data declaration: {}", self.name.id);
        let cst::decls::Data { span, doc, name, attr, params, ctors } = self;

        let ctors = ctors
            .iter()
            .map(|ctor| lower_constructor(ctor, ctx, name, params.len()))
            .collect::<Result<_, _>>()?;

        Ok(ast::Data {
            span: Some(*span),
            doc: doc.lower(ctx)?,
            name: name.lower(ctx)?,
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
) -> Result<ast::Ctor, LoweringError> {
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
                        name: name.lower(ctx)?,
                        args: ast::Args { args: vec![] },
                    }
                } else {
                    return Err(LoweringError::MustProvideArgs {
                        xtor: name.clone(),
                        typ: typ_name.clone(),
                        span: span.to_miette(),
                    });
                }
            }
        };

        Ok(ast::Ctor {
            span: Some(*span),
            doc: doc.lower(ctx)?,
            name: name.lower(ctx)?,
            params,
            typ,
        })
    })
}
