use polarity_lang_ast::ctx::BindContext;
use polarity_lang_parser::cst;

use crate::lower::Lower;

impl Lower for cst::exp::LocalLet {
    type Target = polarity_lang_ast::Exp;

    fn lower(&self, ctx: &mut crate::Ctx) -> crate::LoweringResult<Self::Target> {
        let cst::exp::LocalLet { span, name, typ, bound, body } = self;

        let name = name.lower(ctx)?;
        let typ = typ.lower(ctx)?;
        let bound = bound.lower(ctx)?;

        ctx.bind_single(name.clone(), |ctx| {
            let body = body.lower(ctx)?;
            Ok(polarity_lang_ast::LocalLet {
                span: *span,
                name,
                typ,
                bound,
                body,
                inferred_type: None,
            }
            .into())
        })
    }
}
