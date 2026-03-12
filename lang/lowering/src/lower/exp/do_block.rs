use polarity_lang_ast as ast;
use polarity_lang_ast::ctx::BindContext;
use polarity_lang_miette_util::ToMiette;
use polarity_lang_parser::cst;

use crate::Ctx;
use crate::lower::{Lower, LoweringError, LoweringResult};

impl Lower for cst::exp::DoBlock {
    type Target = ast::Exp;

    fn lower(&self, ctx: &mut Ctx) -> LoweringResult<Self::Target> {
        let cst::exp::DoBlock { span, statements } = self;

        let Some((last, statements)) = statements.split_last() else {
            // The parser's grammar ensures there is at least one statement in the do block.
            return Err(Box::new(LoweringError::Impossible {
                message: "No final return expression in do block.".to_string(),
                span: Some(span.to_miette()),
            }));
        };

        // The last element must be the final return expression.
        let cst::exp::DoStatement::Exp { span: ret_span, exp: ret_exp } = last else {
            // The parser's grammar ensures that the final statement is an expression.
            return Err(Box::new(LoweringError::Impossible {
                message: "Final statement in do block is not an expression.".to_string(),
                span: Some(span.to_miette()),
            }));
        };
        let return_exp = ast::DoStatements::Return {
            span: *ret_span,
            exp: ret_exp.lower(ctx)?,
            inferred_type: None,
        };

        let statements = lower_do_statements(statements, return_exp, ctx)?;

        let block = ast::DoBlock { span: *span, statements, inferred_type: None };
        Ok(ast::Exp::DoBlock(block))
    }
}

fn lower_do_statements(
    statements: &[cst::exp::DoStatement],
    return_exp: ast::DoStatements,
    ctx: &mut Ctx,
) -> LoweringResult<ast::DoStatements> {
    let Some((head, tail)) = statements.split_first() else {
        return Ok(return_exp);
    };

    match head {
        cst::exp::DoStatement::Exp { span, exp } => Ok(ast::DoStatements::Bind {
            span: *span,
            name: ast::VarBind::Wildcard { span: None },
            bound: exp.lower(ctx)?,
            body: Box::new(lower_do_statements(tail, return_exp, ctx)?),
            inferred_type: None,
        }),
        cst::exp::DoStatement::Bind { span, name, bound } => {
            let name = name.lower(ctx)?;
            let bound = bound.lower(ctx)?;

            ctx.bind_single(name.clone(), |ctx| {
                let body = lower_do_statements(tail, return_exp, ctx)?;
                Ok(ast::DoStatements::Bind {
                    span: *span,
                    name,
                    bound,
                    body: Box::new(body),
                    inferred_type: None,
                })
            })
        }
        cst::exp::DoStatement::Let { span, name, typ, bound } => {
            let name = name.lower(ctx)?;
            let typ = typ.lower(ctx)?;
            let bound = bound.lower(ctx)?;

            ctx.bind_single(name.clone(), |ctx| {
                let body = lower_do_statements(tail, return_exp, ctx)?;
                Ok(ast::DoStatements::Let {
                    span: *span,
                    name,
                    typ,
                    bound,
                    body: Box::new(body),
                    inferred_type: None,
                })
            })
        }
    }
}
