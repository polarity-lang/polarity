use polarity_lang_ast as ast;
use polarity_lang_miette_util::ToMiette;
use polarity_lang_parser::cst;

use crate::Ctx;
use crate::lower::{Lower, LoweringError, LoweringResult};

impl Lower for cst::exp::DoBlock {
    type Target = ast::Exp;

    fn lower(&self, ctx: &mut Ctx) -> LoweringResult<Self::Target> {
        let cst::exp::DoBlock { span, statements } = self;

        let mut statements = statements.iter().rev();

        // The last element must be the final return expression.
        let Some(cst::exp::DoStatement::Exp { span: ret_span, exp: ret_exp }) = statements.next()
        else {
            // This is impossible because the parser's grammar ensures that the last statement must
            // always be the return expression.
            return Err(Box::new(LoweringError::Impossible {
                message: "No final return expression in do block.".to_string(),
                span: Some(span.to_miette()),
            }));
        };

        // Build the statement list from bottom to top.
        let mut builder = ast::DoStatements::Return { span: *ret_span, exp: ret_exp.lower(ctx)? };
        for statement in statements {
            match statement {
                cst::exp::DoStatement::Exp { span, exp } => {
                    builder = ast::DoStatements::Bind {
                        span: *span,
                        name: ast::VarBind::Wildcard { span: None },
                        bound: exp.lower(ctx)?,
                        body: Box::new(builder),
                    }
                }
                cst::exp::DoStatement::Bind { span, name, bound } => {
                    builder = ast::DoStatements::Bind {
                        span: *span,
                        name: name.lower(ctx)?,
                        bound: bound.lower(ctx)?,
                        body: Box::new(builder),
                    }
                }
                cst::exp::DoStatement::Let { span, name, typ, bound } => {
                    builder = ast::DoStatements::Let {
                        span: *span,
                        name: name.lower(ctx)?,
                        typ: typ.lower(ctx)?,
                        bound: bound.lower(ctx)?,
                        body: Box::new(builder),
                    }
                }
            }
        }

        let block = ast::DoBlock { span: *span, statements: builder, inferred_type: None };
        Ok(ast::Exp::DoBlock(block))
    }
}
