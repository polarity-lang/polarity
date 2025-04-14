use parser::cst::{self};

use crate::{lower::Lower, Ctx, LoweringResult};

impl Lower for cst::exp::BinOp {
    type Target = ast::Exp;
    fn lower(&self, ctx: &mut Ctx) -> LoweringResult<Self::Target> {
        let cst::exp::BinOp { span, operator, lhs, rhs } = self;

        let (id, _url) = ctx.symbol_table.lookup_operator(operator)?;
        let (_, uri) = ctx.symbol_table.lookup(id)?;

        Ok(ast::TypCtor {
            span: Some(*span),
            name: ast::IdBound { span: Some(*span), id: id.id.clone(), uri: uri.clone() },
            args: ast::Args {
                args: vec![
                    ast::Arg::UnnamedArg { arg: lhs.lower(ctx)?, erased: false },
                    ast::Arg::UnnamedArg { arg: rhs.lower(ctx)?, erased: false },
                ],
            },
            is_bin_op: Some(operator.id.clone()),
        }
        .into())
    }
}
