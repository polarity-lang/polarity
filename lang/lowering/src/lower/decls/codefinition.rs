use miette_util::ToMiette;
use parser::cst::{self};

use super::super::*;
use super::lower_telescope;

impl Lower for cst::decls::Codef {
    type Target = ast::Codef;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        log::trace!("Lowering codefinition: {}", self.name.id);

        let cst::decls::Codef { span, doc, name, attr, params, typ, cases, .. } = self;

        lower_telescope(params, ctx, |ctx, params| {
            let typ = typ.lower(ctx)?;
            let typ_ctor = typ
                .to_typctor()
                .ok_or(LoweringError::ExpectedTypCtor { span: span.to_miette() })?;
            Ok(ast::Codef {
                span: Some(*span),
                doc: doc.lower(ctx)?,
                name: name.lower(ctx)?,
                attr: attr.lower(ctx)?,
                params,
                typ: typ_ctor,
                cases: cases.lower(ctx)?,
            })
        })
    }
}
