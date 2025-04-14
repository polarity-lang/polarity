use miette_util::ToMiette;
use parser::cst;

use crate::{lower::Lower, Ctx, DeclMeta, LoweringError, LoweringResult};

use super::args::lower_args;

impl Lower for cst::exp::DotCall {
    type Target = ast::Exp;

    fn lower(&self, ctx: &mut Ctx) -> LoweringResult<Self::Target> {
        let cst::exp::DotCall { span, exp, name, args } = self;

        let (meta, name) = ctx.symbol_table.lookup(name)?;

        match meta.clone() {
            DeclMeta::Dtor { params, .. } => Ok(ast::Exp::DotCall(ast::DotCall {
                span: Some(*span),
                kind: ast::DotCallKind::Destructor,
                exp: exp.lower(ctx)?,
                name,
                args: lower_args(*span, args, params, ctx)?,
                inferred_type: None,
            })),
            DeclMeta::Def { params, .. } => Ok(ast::Exp::DotCall(ast::DotCall {
                span: Some(*span),
                kind: ast::DotCallKind::Definition,
                exp: exp.lower(ctx)?,
                name,
                args: lower_args(*span, args, params, ctx)?,
                inferred_type: None,
            })),
            _ => Err(LoweringError::CannotUseAsDotCall { name, span: span.to_miette() }),
        }
    }
}
