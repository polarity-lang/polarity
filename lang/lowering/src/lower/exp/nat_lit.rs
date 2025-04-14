use miette_util::ToMiette;
use num_bigint::BigUint;
use parser::cst::{self, Ident};

use crate::{lower::Lower, Ctx, DeclMeta, LoweringError, LoweringResult};

impl Lower for cst::exp::NatLit {
    type Target = ast::Exp;

    fn lower(&self, ctx: &mut Ctx) -> LoweringResult<Self::Target> {
        let cst::exp::NatLit { span, val } = self;

        // We have to check whether "Z" is declared as a constructor or codefinition.
        // We assume that if Z exists, then S exists as well and is of the same kind.
        let (z_kind, name) =
            ctx.symbol_table.lookup(&Ident { span: *span, id: "Z".to_string() }).map_err(|_| {
                LoweringError::NatLiteralCannotBeDesugared { span: span.to_miette() }
            })?;
        let call_kind = match z_kind {
            DeclMeta::Codef { .. } => ast::CallKind::Codefinition,
            DeclMeta::Ctor { .. } => ast::CallKind::Constructor,
            _ => {
                return Err(
                    LoweringError::NatLiteralCannotBeDesugared { span: span.to_miette() }.into()
                )
            }
        };

        let mut out = ast::Exp::Call(ast::Call {
            span: Some(*span),
            kind: call_kind,
            name: name.clone(),
            args: ast::Args { args: vec![] },
            inferred_type: None,
        });

        let mut i = BigUint::from(0usize);

        while &i != val {
            i += 1usize;
            out = ast::Exp::Call(ast::Call {
                span: Some(*span),
                kind: call_kind,
                name: ast::IdBound { span: Some(*span), id: "S".to_owned(), uri: name.uri.clone() },
                args: ast::Args {
                    args: vec![ast::Arg::UnnamedArg { arg: Box::new(out), erased: false }],
                },
                inferred_type: None,
            });
        }

        Ok(out)
    }
}
