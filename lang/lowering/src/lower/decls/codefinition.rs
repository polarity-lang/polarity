use polarity_lang_ast::IdBind;
use polarity_lang_miette_util::ToMiette;
use polarity_lang_parser::cst::{self};

use super::super::*;
use super::lower_telescope;

impl Lower for cst::decls::Codef {
    type Target = polarity_lang_ast::Codef;

    fn lower(&self, ctx: &mut Ctx) -> LoweringResult<Self::Target> {
        log::trace!("Lowering codefinition: {}", self.name.id);

        let cst::decls::Codef { span, doc, name, attr, params, typ, cases, .. } = self;

        lower_telescope(params, ctx, |ctx, params| {
            let typ = typ.lower(ctx)?;
            let typ_ctor = typ
                .to_typctor()
                .ok_or(LoweringError::ExpectedTypCtor { span: span.to_miette() })?;
            Ok(polarity_lang_ast::Codef {
                span: Some(*span),
                doc: doc.lower(ctx)?,
                name: IdBind { span: Some(name.span), id: name.id.clone() },
                attr: attr.lower(ctx)?,
                params,
                typ: typ_ctor,
                cases: cases.lower(ctx)?,
            })
        })
    }
}
