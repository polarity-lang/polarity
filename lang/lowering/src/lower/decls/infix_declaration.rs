use ast::ctx::BindContext;
use parser::cst;

use super::super::*;

impl Lower for cst::decls::Infix {
    type Target = ast::Infix;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::decls::Infix { span, doc, lhs, rhs } = self;

        let left_binder = lhs.lhs.lower(ctx)?;
        let right_binder = lhs.rhs.lower(ctx)?;

        ctx.bind_iter([left_binder, right_binder].into_iter(), |ctx| rhs.lower(ctx))?;

        Ok(ast::Infix {
            span: Some(*span),
            doc: doc.lower(ctx)?,
            attr: Default::default(),
            lhs: lhs.operator.id.clone(),
            rhs: rhs.name.id.clone(),
        })
    }
}
