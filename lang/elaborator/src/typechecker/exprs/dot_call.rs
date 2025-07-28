//! Bidirectional type checker

use crate::conversion_checking::convert;
use crate::normalizer::env::ToEnv;
use crate::normalizer::normalize::Normalize;
use crate::result::TcResult;
use crate::typechecker::erasure;
use crate::typechecker::type_info_table::DtorMeta;
use ast::*;

use super::super::ctx::*;
use super::CheckInfer;
use super::ExpectType;
use super::check_args;

impl CheckInfer for DotCall {
    /// The *checking* rule for dotcalls is:
    /// ```text
    ///                 ...
    ///           ──────────────────
    ///            P, Γ ⊢ e.Dσ ⇐ τ
    /// ```
    fn check(&self, ctx: &mut Ctx, t: &Exp) -> TcResult<Self> {
        let inferred_term = self.infer(ctx)?;
        let inferred_typ = inferred_term.expect_typ()?;
        convert(&ctx.vars, &mut ctx.meta_vars, inferred_typ, t, &self.span())?;
        Ok(inferred_term)
    }

    /// The *inference* rule for dotcalls is:
    /// ```text
    ///                 ...
    ///           ──────────────────
    ///            P, Γ ⊢ e.Dσ ⇒ ...
    /// ```
    fn infer(&self, ctx: &mut Ctx) -> TcResult<Self> {
        let DotCall { span, kind, exp, name, args, .. } = self;
        let DtorMeta { params, self_param, ret_typ, .. } =
            &ctx.type_info_table.lookup_dtor_or_def(&name.clone())?;

        let mut args_out = check_args(args, &name.clone(), ctx, params, *span)?;

        let self_param_out = self_param
            .typ
            .subst_new(&mut vec![params.params.clone()].into(), &Subst::from_args(&vec![args.args.clone()]))
            .to_exp();
        let self_param_nf = self_param_out.normalize(&ctx.type_info_table, &mut ctx.env())?;

        let exp_out = exp.check(ctx, &self_param_nf)?;

        let subst = Subst::from_exps(&vec![args.to_exps(), vec![exp.clone()]]);
        let typ_out = ret_typ
            .subst_new(&mut vec![params.params.clone(), vec![self_param.to_param()]].into(), &subst);
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
