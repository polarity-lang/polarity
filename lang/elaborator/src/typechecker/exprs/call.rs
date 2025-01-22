//! Bidirectional type checker

use crate::conversion_checking::convert;
use crate::normalizer::env::ToEnv;
use crate::normalizer::normalize::Normalize;
use crate::result::TcResult;
use crate::typechecker::erasure;
use crate::typechecker::type_info_table::CtorMeta;
use ast::*;

use super::super::ctx::*;
use super::check_args;
use super::CheckInfer;
use crate::result::TypeError;

impl CheckInfer for Call {
    /// The *checking* rule for calls is:
    /// ```text
    ///                 ...
    ///           ──────────────────
    ///            P, Γ ⊢ Cσ ⇐ τ
    /// ```
    fn check(&self, ctx: &mut Ctx, t: &Exp) -> TcResult<Self> {
        let inferred_term = self.infer(ctx)?;
        let inferred_typ = inferred_term.typ().ok_or(TypeError::Impossible {
            message: "Expected inferred type".to_owned(),
            span: None,
        })?;
        convert(ctx.vars.clone(), &mut ctx.meta_vars, inferred_typ, t, &self.span())?;
        Ok(inferred_term)
    }
    /// The *inference* rule for calls is:
    /// ```text
    ///                 ...
    ///           ──────────────────
    ///            P, Γ ⊢ Cσ ⇒ ...
    /// ```
    fn infer(&self, ctx: &mut Ctx) -> TcResult<Self> {
        let Call { span, kind, name, args, .. } = self;

        match kind {
            CallKind::Codefinition | CallKind::Constructor => {
                let CtorMeta { params, typ, .. } =
                    &ctx.type_info_table.lookup_ctor_or_codef(&name.clone())?;
                let mut args_out = check_args(args, &name.clone(), ctx, params, *span)?;
                let typ_out = typ
                    .subst_under_ctx(vec![params.params.clone()].into(), &vec![args.args.clone()])
                    .to_exp();
                let typ_nf = typ_out.normalize(&ctx.type_info_table, &mut ctx.env())?;

                erasure::mark_erased_args(params, &mut args_out);

                Ok(Call {
                    span: *span,
                    kind: *kind,
                    name: name.clone(),
                    args: args_out,
                    inferred_type: Some(typ_nf),
                })
            }
            CallKind::LetBound => {
                let Let { params, typ, .. } = ctx.type_info_table.lookup_let(&name.clone())?;
                let params = params.clone();
                let typ = typ.clone();
                let mut args_out = check_args(args, &name.clone(), ctx, &params, *span)?;
                let typ_out = typ
                    .subst_under_ctx(vec![params.params.clone()].into(), &vec![args.args.clone()]);
                let typ_nf = typ_out.normalize(&ctx.type_info_table, &mut ctx.env())?;

                erasure::mark_erased_args(&params, &mut args_out);

                Ok(Call {
                    span: *span,
                    kind: *kind,
                    name: name.clone(),
                    args: args_out,
                    inferred_type: Some(typ_nf),
                })
            }
        }
    }
}
