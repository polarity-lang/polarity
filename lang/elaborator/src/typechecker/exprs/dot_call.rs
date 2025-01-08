//! Bidirectional type checker

use crate::normalizer::env::ToEnv;
use crate::normalizer::normalize::Normalize;
use crate::typechecker::erasure;
use crate::typechecker::type_info_table::DtorMeta;
use ast::*;

use super::super::ctx::*;
use super::super::util::*;
use super::check_args;
use super::CheckInfer;
use crate::result::TypeError;

impl CheckInfer for DotCall {
    /// The *checking* rule for dotcalls is:
    /// ```text
    ///                 ...
    ///           ──────────────────
    ///            P, Γ ⊢ e.Dσ ⇐ τ
    /// ```
    fn check(&self, ctx: &mut Ctx, t: &Exp) -> Result<Self, TypeError> {
        let inferred_term = self.infer(ctx)?;
        let inferred_typ = inferred_term.typ().ok_or(TypeError::Impossible {
            message: "Expected inferred type".to_owned(),
            span: None,
        })?;
        convert(ctx.levels(), &mut ctx.meta_vars, inferred_typ, t, &self.span())?;
        Ok(inferred_term)
    }

    /// The *inference* rule for dotcalls is:
    /// ```text
    ///                 ...
    ///           ──────────────────
    ///            P, Γ ⊢ e.Dσ ⇒ ...
    /// ```
    fn infer(&self, ctx: &mut Ctx) -> Result<Self, TypeError> {
        let DotCall { span, kind, exp, name, args, .. } = self;
        let DtorMeta { params, self_param, ret_typ, .. } =
            &ctx.type_info_table.lookup_dtor_or_def(&name.clone())?;

        let mut args_out = check_args(args, &name.clone(), ctx, params, *span)?;

        let self_param_out = self_param
            .typ
            .subst_under_ctx(vec![params.len()].into(), &vec![args.args.clone()])
            .to_exp();
        let self_param_nf = self_param_out.normalize(&ctx.type_info_table, &mut ctx.env())?;

        let exp_out = exp.check(ctx, &self_param_nf)?;

        let subst = vec![args.to_exps(), vec![exp.clone()]];
        let typ_out = ret_typ.subst_under_ctx(vec![params.len(), 1].into(), &subst);
        let typ_out_nf = typ_out.normalize(&ctx.type_info_table, &mut ctx.env())?;

        erasure::mark_erased_args(params, &mut args_out);

        Ok(DotCall {
            span: *span,
            kind: *kind,
            exp: exp_out,
            name: name.to_owned(),
            args: args_out,
            inferred_type: Some(typ_out_nf),
        })
    }
}
