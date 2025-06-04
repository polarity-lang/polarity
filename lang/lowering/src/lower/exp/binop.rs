use parser::cst::{self};

use crate::{Ctx, LoweringResult, lower::Lower};

impl Lower for cst::exp::BinOp {
    type Target = ast::Exp;
    fn lower(&self, ctx: &mut Ctx) -> LoweringResult<Self::Target> {
        let cst::exp::BinOp { span, lhs, rhs } = self;

        match rhs.as_slice() {
            [] => {
                let lowered = lhs.lower(ctx)?;
                Ok(*lowered)
            }
            [(operator, rhs), tail @ ..] => {
                let (id, _url) = ctx.symbol_table.lookup_operator(operator)?;
                let (_, name) = ctx.symbol_table.lookup(id)?;

                let new_bin_op =
                    cst::exp::BinOp { span: *span, lhs: Box::new(rhs.clone()), rhs: tail.to_vec() };
                let rhs_lowered = new_bin_op.lower(ctx)?;

                Ok(ast::TypCtor {
                    span: Some(*span),
                    name,
                    args: ast::Args {
                        args: vec![
                            ast::Arg::UnnamedArg { arg: lhs.lower(ctx)?, erased: false },
                            ast::Arg::UnnamedArg { arg: Box::new(rhs_lowered), erased: false },
                        ],
                    },
                    is_bin_op: Some(operator.id.clone()),
                }
                .into())
            }
        }
    }
}
