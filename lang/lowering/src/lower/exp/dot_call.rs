use polarity_lang_miette_util::ToMiette;
use polarity_lang_parser::cst;

use crate::{Ctx, DeclMeta, LoweringError, LoweringResult, expect_ident, lower::Lower};

use super::args::lower_args;

impl Lower for cst::exp::DotCall {
    type Target = polarity_lang_ast::Exp;

    fn lower(&self, ctx: &mut Ctx) -> LoweringResult<Self::Target> {
        let cst::exp::DotCall { span, exp, name, args } = self;

        let name = expect_ident(name.clone())?;
        let (meta, name) = ctx.symbol_table.lookup(&name)?;

        match meta.clone() {
            DeclMeta::Dtor { params, .. } => {
                Ok(polarity_lang_ast::Exp::DotCall(polarity_lang_ast::DotCall {
                    span: Some(*span),
                    kind: polarity_lang_ast::DotCallKind::Destructor,
                    exp: exp.lower(ctx)?,
                    name,
                    args: lower_args(*span, args, params, ctx)?,
                    inferred_type: None,
                }))
            }
            DeclMeta::Def { params, .. } => {
                Ok(polarity_lang_ast::Exp::DotCall(polarity_lang_ast::DotCall {
                    span: Some(*span),
                    kind: polarity_lang_ast::DotCallKind::Definition,
                    exp: exp.lower(ctx)?,
                    name,
                    args: lower_args(*span, args, params, ctx)?,
                    inferred_type: None,
                }))
            }
            _ => Err(LoweringError::CannotUseAsDotCall { name, span: span.to_miette() }.into()),
        }
    }
}
