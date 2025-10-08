use num_bigint::BigUint;
use polarity_lang_miette_util::ToMiette;
use polarity_lang_parser::cst::{self, Ident};

use crate::{Ctx, DeclMeta, LoweringError, LoweringResult, lower::Lower};

impl Lower for cst::exp::NatLit {
    type Target = polarity_lang_ast::Exp;

    fn lower(&self, ctx: &mut Ctx) -> LoweringResult<Self::Target> {
        let cst::exp::NatLit { span, val } = self;

        // We have to check whether "Z" is declared as a constructor or codefinition.
        // We assume that if Z exists, then S exists as well and is of the same kind.
        let (z_kind, name) =
            ctx.symbol_table.lookup(&Ident { span: *span, id: "Z".to_string() }).map_err(|_| {
                LoweringError::NatLiteralCannotBeDesugared { span: span.to_miette() }
            })?;
        let call_kind = match z_kind {
            DeclMeta::Codef { .. } => polarity_lang_ast::CallKind::Codefinition,
            DeclMeta::Ctor { .. } => polarity_lang_ast::CallKind::Constructor,
            _ => {
                return Err(
                    LoweringError::NatLiteralCannotBeDesugared { span: span.to_miette() }.into()
                );
            }
        };

        let mut out = polarity_lang_ast::Exp::Call(polarity_lang_ast::Call {
            span: Some(*span),
            kind: call_kind,
            name: name.clone(),
            args: polarity_lang_ast::Args { args: vec![] },
            inferred_type: None,
        });

        let mut i = BigUint::from(0usize);

        while &i != val {
            i += 1usize;
            out = polarity_lang_ast::Exp::Call(polarity_lang_ast::Call {
                span: Some(*span),
                kind: call_kind,
                name: polarity_lang_ast::IdBound {
                    span: Some(*span),
                    id: "S".to_owned(),
                    uri: name.uri.clone(),
                },
                args: polarity_lang_ast::Args {
                    args: vec![polarity_lang_ast::Arg::UnnamedArg {
                        arg: Box::new(out),
                        erased: false,
                    }],
                },
                inferred_type: None,
            });
        }

        Ok(out)
    }
}
