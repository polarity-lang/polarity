use crate::lower::*;

use polarity_lang_ast::IdBind;
use polarity_lang_parser::cst;

impl Lower for cst::decls::Note {
    type Target = polarity_lang_ast::Note;

    fn lower(&self, ctx: &mut Ctx) -> LoweringResult<Self::Target> {
        let cst::decls::Note { span, doc, name, attr } = self;

        Ok(Self::Target {
            span: Some(*span),
            doc: doc.lower(ctx)?,
            name: IdBind { span: Some(name.span), id: name.id.clone() },
            attr: attr.lower(ctx)?,
        })
    }
}
