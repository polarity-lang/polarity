use polarity_lang_parser::cst::{self};

use crate::{Ctx, LoweringResult, lower::Lower};

impl Lower for cst::exp::BinOp {
    type Target = polarity_lang_ast::Exp;
    fn lower(&self, ctx: &mut Ctx) -> LoweringResult<Self::Target> {
        let cst::exp::BinOp { span, lhs, rhs } = self;

        // Note: Currently all binary operators are lowered as right-associative.
        // The plan is to add associativity information to infix declarations so
        // that we can lower accordingly.
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

                Ok(polarity_lang_ast::TypCtor {
                    span: Some(*span),
                    name,
                    args: polarity_lang_ast::Args {
                        args: vec![
                            polarity_lang_ast::Arg::UnnamedArg {
                                arg: lhs.lower(ctx)?,
                                erased: false,
                            },
                            polarity_lang_ast::Arg::UnnamedArg {
                                arg: Box::new(rhs_lowered),
                                erased: false,
                            },
                        ],
                    },
                    is_bin_op: Some(operator.id.clone()),
                }
                .into())
            }
        }
    }
}
