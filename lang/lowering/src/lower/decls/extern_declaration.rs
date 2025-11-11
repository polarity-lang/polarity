use polarity_lang_ast::IdBind;
use polarity_lang_parser::cst::{self};

use super::super::*;
use super::lower_telescope;

impl Lower for cst::decls::Extern {
    type Target = polarity_lang_ast::Extern;

    fn lower(&self, ctx: &mut Ctx) -> LoweringResult<Self::Target> {
        log::trace!("Lowering extern declaration: {}", self.name.id);

        let cst::decls::Extern { span, doc, name, attr, params, typ } = self;

        lower_telescope(params, ctx, |ctx, params| {
            Ok(polarity_lang_ast::Extern {
                span: Some(*span),
                doc: doc.lower(ctx)?,
                name: IdBind { span: Some(name.span), id: name.id.clone() },
                attr: attr.lower(ctx)?,
                params,
                typ: typ.lower(ctx)?,
            })
        })
    }
}
