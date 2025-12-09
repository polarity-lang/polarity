use polarity_lang_miette_util::ToMiette;
use polarity_lang_parser::cst::{self, Ident, exp::LiteralKind};

use crate::{Ctx, DeclMeta, LoweringError, LoweringResult, lower::Lower};

impl Lower for cst::exp::Literal {
    type Target = polarity_lang_ast::Exp;

    fn lower(&self, ctx: &mut Ctx) -> LoweringResult<Self::Target> {
        let cst::exp::Literal { span, kind } = self;

        // Get the correct name we are binding to
        let type_id = match kind {
            LiteralKind::I64(_) => "I64".to_owned(),
            LiteralKind::F64(_) => "F64".to_owned(),
            LiteralKind::Char { .. } => "Char".to_owned(),
            LiteralKind::String { .. } => "String".to_owned(),
        };
        let type_ident = Ident {
            // Use literal's span as synthetic span
            span: *span,
            id: type_id,
        };

        // Lookup what type is in scope
        let (type_decl, type_name) = ctx.symbol_table.lookup(&type_ident)?;

        // Lower to AST variant
        let ast_literal = match kind {
            LiteralKind::I64(v) => polarity_lang_ast::LiteralKind::I64(*v),
            LiteralKind::F64(v) => polarity_lang_ast::LiteralKind::F64(*v),
            LiteralKind::Char { original, unescaped } => polarity_lang_ast::LiteralKind::Char {
                original: original.clone(),
                unescaped: unescaped.clone(),
            },
            LiteralKind::String { original, unescaped } => polarity_lang_ast::LiteralKind::String {
                original: original.clone(),
                unescaped: unescaped.clone(),
            },
        };

        match type_decl {
            DeclMeta::Extern { params } if params.is_empty() => {
                Ok(polarity_lang_ast::Exp::Literal(polarity_lang_ast::Literal {
                    span: Some(*span),
                    kind: ast_literal,
                    inferred_type: Box::new(polarity_lang_ast::Exp::Call(
                        polarity_lang_ast::Call {
                            // Use literal's span as synthetic span
                            span: Some(*span),
                            name: type_name,
                            kind: polarity_lang_ast::CallKind::Extern,
                            args: polarity_lang_ast::Args { args: vec![] },
                            inferred_type: None,
                        },
                    )),
                }))
            }
            _ => Err(Box::new(LoweringError::InvalidTypeDeclForLiteral {
                span: span.to_miette(),
                typ: type_name.id,
            })),
        }
    }
}
