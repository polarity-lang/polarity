use crate::lower::*;

use ast::IdBind;
use parser::cst;

impl Lower for cst::decls::Note {
    type Target = ast::Note;

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
