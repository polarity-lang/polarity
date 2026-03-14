use polarity_lang_ast::{Args, Call, CallKind, TypCtor};
use polarity_lang_miette_util::ToMiette;
use polarity_lang_parser::cst::{self};

use crate::{Ctx, DeclMeta, LoweringError, LoweringResult, lower::Lower};

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
                let (meta, name) = ctx.symbol_table.lookup(id)?;
                let meta = meta.clone();

                let new_bin_op =
                    cst::exp::BinOp { span: *span, lhs: Box::new(rhs.clone()), rhs: tail.to_vec() };
                let rhs_lowered = new_bin_op.lower(ctx)?;
                let args = Args {
                    args: vec![
                        polarity_lang_ast::Arg::UnnamedArg { arg: lhs.lower(ctx)?, erased: false },
                        polarity_lang_ast::Arg::UnnamedArg {
                            arg: Box::new(rhs_lowered),
                            erased: false,
                        },
                    ],
                };

                if let DeclMeta::Data { .. } | DeclMeta::Codata { .. } = meta {
                    return Ok(TypCtor {
                        span: Some(*span),
                        name,
                        args,
                        is_bin_op: Some(operator.id.clone()),
                    }
                    .into());
                };

                let kind = match meta {
                    DeclMeta::Ctor { .. } => CallKind::Constructor,
                    DeclMeta::Codef { .. } => CallKind::Codefinition,
                    DeclMeta::Let { .. } => CallKind::LetBound,
                    DeclMeta::Extern { .. } => CallKind::Extern,
                    _ => {
                        return Err(LoweringError::Impossible {
                            message: "Unexpected declaration kind in infix lowering".to_owned(),
                            span: Some(span.to_miette()),
                        }
                        .into());
                    }
                };

                Ok(Call {
                    span: Some(*span),
                    kind,
                    name,
                    args,
                    is_bin_op: Some(operator.id.clone()),
                    inferred_type: None,
                }
                .into())
            }
        }
    }
}
