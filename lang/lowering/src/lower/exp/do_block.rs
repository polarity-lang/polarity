use polarity_lang_ast as ast;
use polarity_lang_miette_util::ToMiette;
use polarity_lang_parser::cst;

use crate::Ctx;
use crate::lower::{Lower, LoweringError, LoweringResult};

impl Lower for cst::exp::DoBlock {
    type Target = ast::Exp;

    fn lower(&self, ctx: &mut Ctx) -> LoweringResult<Self::Target> {
        let cst::exp::DoBlock { span, statements } = self;

        let Some(cst::exp::DoStatement::Exp { span: _, exp: return_exp }) = statements.last()
        else {
            // This is impossible because the parser's grammar ensures that the last statement must
            // always be the return expression.
            return Err(Box::new(LoweringError::Impossible {
                message: "No final return expression in do block.".to_string(),
                span: Some(span.to_miette()),
            }));
        };
        let return_exp = return_exp.lower(ctx)?;

        let mut bindings = Vec::new();
        for s in statements.iter().take(statements.len() - 1) {
            let binding = match s {
                cst::exp::DoStatement::Exp { span, exp } => ast::DoBinding::Bind {
                    span: *span,
                    var: ast::VarBind::Wildcard { span: None },
                    exp: exp.lower(ctx)?,
                },
                cst::exp::DoStatement::Bind { span, var, exp } => {
                    ast::DoBinding::Bind { span: *span, var: var.lower(ctx)?, exp: exp.lower(ctx)? }
                }
                cst::exp::DoStatement::Let { span, var, typ, exp } => ast::DoBinding::Let {
                    span: *span,
                    var: var.lower(ctx)?,
                    typ: typ.lower(ctx)?,
                    exp: exp.lower(ctx)?,
                },
            };
            bindings.push(binding);
        }

        Ok(ast::Exp::DoBlock(ast::DoBlock {
            span: *span,
            bindings,
            return_exp,
            inferred_type: None,
        }))
    }
}
