use ast::{IdBound, TypeUniv, VarBound, Variable};
use miette_util::ToMiette;
use parser::cst;

use crate::{lower::Lower, Ctx, DeclMeta, LoweringError, LoweringResult};

use super::args::lower_args;

impl Lower for cst::exp::Call {
    type Target = ast::Exp;

    fn lower(&self, ctx: &mut Ctx) -> LoweringResult<Self::Target> {
        let cst::exp::Call { span, name, args } = self;

        // The type universe "Type" is treated as an ordinary call in the lexer and parser.
        // For this reason we have to special case the logic for lowering the type universe here.
        if name.id == "Type" {
            if !args.is_empty() {
                return Err(LoweringError::TypeUnivArgs { span: span.to_miette() });
            }
            return Ok(TypeUniv { span: Some(*span) }.into());
        }

        // If we find the identifier in the local context then we have to lower
        // it to a variable.
        if let Some(idx) = ctx.lookup_local(name) {
            let name = VarBound { span: Some(name.span), id: name.id.clone() };
            return Ok(ast::Exp::Variable(Variable {
                span: Some(*span),
                idx,
                name,
                inferred_type: None,
            }));
        }

        // If we find the identifier in the global context then we have to lower
        // it to a call or a type constructor.
        let (meta, uri) = ctx.symbol_table.lookup(name)?;
        match meta {
            DeclMeta::Data { params, .. } | DeclMeta::Codata { params, .. } => {
                let name = IdBound { span: Some(name.span), id: name.id.clone(), uri: uri.clone() };
                Ok(ast::Exp::TypCtor(ast::TypCtor {
                    span: Some(*span),
                    name,
                    args: lower_args(*span, args, params.clone(), ctx)?,
                    is_bin_op: None,
                }))
            }
            DeclMeta::Def { .. } | DeclMeta::Dtor { .. } => {
                Err(LoweringError::MustUseAsDotCall { name: name.clone(), span: span.to_miette() })
            }
            DeclMeta::Ctor { params, .. } => {
                let name = IdBound { span: Some(name.span), id: name.id.clone(), uri: uri.clone() };
                Ok(ast::Exp::Call(ast::Call {
                    span: Some(*span),
                    kind: ast::CallKind::Constructor,
                    name,
                    args: lower_args(*span, args, params.clone(), ctx)?,
                    inferred_type: None,
                }))
            }
            DeclMeta::Codef { params, .. } => {
                let name = IdBound { span: Some(name.span), id: name.id.clone(), uri: uri.clone() };
                Ok(ast::Exp::Call(ast::Call {
                    span: Some(*span),
                    kind: ast::CallKind::Codefinition,
                    name,
                    args: lower_args(*span, args, params.clone(), ctx)?,
                    inferred_type: None,
                }))
            }
            DeclMeta::Let { params, .. } => {
                let name = IdBound { span: Some(name.span), id: name.id.clone(), uri: uri.clone() };
                Ok(ast::Exp::Call(ast::Call {
                    span: Some(*span),
                    kind: ast::CallKind::LetBound,
                    name,
                    args: lower_args(*span, args, params.clone(), ctx)?,
                    inferred_type: None,
                }))
            }
        }
    }
}
