use polarity_lang_parser::cst::{self, Ident};

use crate::{Ctx, DeclMeta, LoweringResult, lower::Lower};

impl Lower for cst::exp::StrLit {
    type Target = polarity_lang_ast::Exp;

    fn lower(&self, ctx: &mut Ctx) -> LoweringResult<Self::Target> {
        let cst::exp::StrLit { span, original, unescaped } = self;

        // Lookup what "String" type is in scope
        let string_ident = Ident {
            // Use literal's span as dummy
            span: *span,
            id: "String".to_owned(),
        };
        let (string_decl, string_name) = ctx.symbol_table.lookup(&string_ident)?;
        match string_decl {
            DeclMeta::Extern { params } if params.is_empty() => {
                Ok(polarity_lang_ast::Exp::Literal(polarity_lang_ast::Literal {
                    span: Some(*span),
                    kind: polarity_lang_ast::LiteralKind::Str {
                        original: original.clone(),
                        unescaped: unescaped.clone(),
                    },
                    inferred_type: Box::new(polarity_lang_ast::Exp::TypCtor(
                        polarity_lang_ast::TypCtor {
                            // Use literal's span as dummy
                            span: Some(*span),
                            name: string_name,
                            args: polarity_lang_ast::Args { args: vec![] },
                            is_bin_op: None,
                        },
                    )),
                }))
            }
            _ => todo!(),
        }
    }
}
