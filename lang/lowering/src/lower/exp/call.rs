use polarity_lang_ast::{TypeUniv, VarBound, Variable};
use polarity_lang_miette_util::ToMiette;
use polarity_lang_parser::cst;

use crate::{Ctx, DeclMeta, LoweringError, LoweringResult, expect_ident, lower::Lower};

use super::args::lower_args;

impl Lower for cst::exp::Call {
    type Target = polarity_lang_ast::Exp;

    fn lower(&self, ctx: &mut Ctx) -> LoweringResult<Self::Target> {
        let cst::exp::Call { span, name, args } = self;

        // The type universe "Type" is treated as an ordinary call in the lexer and parser.
        // For this reason we have to special case the logic for lowering the type universe here.
        if name.id == "Type" {
            if !args.is_empty() {
                return Err(LoweringError::TypeUnivArgs { span: span.to_miette() }.into());
            }
            return Ok(TypeUniv { span: Some(*span) }.into());
        }

        // If we find the identifier in the local context then we have to lower
        // it to a variable.
        let name = expect_ident(name.clone())?;
        if let Some(idx) = ctx.lookup_local(&name) {
            let name = VarBound { span: Some(name.span), id: name.id.clone() };
            return Ok(polarity_lang_ast::Exp::Variable(Variable {
                span: Some(*span),
                idx,
                name,
                inferred_type: None,
                erased: false,
            }));
        }

        // If we find the identifier in the global context then we have to lower
        // it to a call or a type constructor.
        let (meta, name) = ctx.symbol_table.lookup(&name)?;
        match meta {
            DeclMeta::Data { params, .. } | DeclMeta::Codata { params, .. } => {
                Ok(polarity_lang_ast::Exp::TypCtor(polarity_lang_ast::TypCtor {
                    span: Some(*span),
                    name,
                    args: lower_args(*span, args, params.clone(), ctx)?,
                    is_bin_op: None,
                }))
            }
            DeclMeta::Def { .. } | DeclMeta::Dtor { .. } => {
                Err(LoweringError::MustUseAsDotCall { name: name.clone(), span: span.to_miette() }
                    .into())
            }
            DeclMeta::Ctor { params, .. } => {
                Ok(polarity_lang_ast::Exp::Call(polarity_lang_ast::Call {
                    span: Some(*span),
                    kind: polarity_lang_ast::CallKind::Constructor,
                    name,
                    args: lower_args(*span, args, params.clone(), ctx)?,
                    inferred_type: None,
                }))
            }
            DeclMeta::Codef { params, .. } => {
                Ok(polarity_lang_ast::Exp::Call(polarity_lang_ast::Call {
                    span: Some(*span),
                    kind: polarity_lang_ast::CallKind::Codefinition,
                    name,
                    args: lower_args(*span, args, params.clone(), ctx)?,
                    inferred_type: None,
                }))
            }
            DeclMeta::Let { params, .. } => {
                Ok(polarity_lang_ast::Exp::Call(polarity_lang_ast::Call {
                    span: Some(*span),
                    kind: polarity_lang_ast::CallKind::LetBound,
                    name,
                    args: lower_args(*span, args, params.clone(), ctx)?,
                    inferred_type: None,
                }))
            }
            DeclMeta::Extern { params, .. } => {
                Ok(polarity_lang_ast::Exp::Call(polarity_lang_ast::Call {
                    span: Some(*span),
                    kind: polarity_lang_ast::CallKind::Extern,
                    name,
                    args: lower_args(*span, args, params.clone(), ctx)?,
                    inferred_type: None,
                }))
            }
            DeclMeta::Note => {
                Err(LoweringError::MisusedNote { span: span.to_miette(), name: name.id }.into())
            }
        }
    }
}
