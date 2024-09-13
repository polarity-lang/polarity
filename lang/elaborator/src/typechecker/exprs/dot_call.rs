//! Bidirectional type checker

use std::rc::Rc;

use crate::normalizer::env::ToEnv;
use crate::normalizer::normalize::Normalize;
use syntax::ast::*;

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
    fn check(&self, prg: &Module, ctx: &mut Ctx, t: Rc<Exp>) -> Result<Self, TypeError> {
        let inferred_term = self.infer(prg, ctx)?;
        let inferred_typ = inferred_term.typ().ok_or(TypeError::Impossible {
            message: "Expected inferred type".to_owned(),
            span: None,
        })?;
        convert(ctx.levels(), &mut ctx.meta_vars, inferred_typ, &t)?;
        Ok(inferred_term)
    }

    /// The *inference* rule for dotcalls is:
    /// ```text
    ///                 ...
    ///           ──────────────────
    ///            P, Γ ⊢ e.Dσ ⇒ ...
    /// ```
    fn infer(&self, prg: &Module, ctx: &mut Ctx) -> Result<Self, TypeError> {
        let DotCall { span, kind, exp, name, args, .. } = self;
        let Dtor { name, params, self_param, ret_typ, .. } = &prg.dtor_or_def(name, *span)?;

        let args_out = check_args(args, prg, name, ctx, params, *span)?;

        let self_param_out = self_param
            .typ
            .subst_under_ctx(vec![params.len()].into(), &vec![args.args.clone()])
            .to_exp();
        let self_param_nf = self_param_out.normalize(prg, &mut ctx.env())?;

        let exp_out = exp.check(prg, ctx, self_param_nf)?;

        let subst = vec![args.to_exps(), vec![exp.clone()]];
        let typ_out = ret_typ.subst_under_ctx(vec![params.len(), 1].into(), &subst);
        let typ_out_nf = typ_out.normalize(prg, &mut ctx.env())?;

        Ok(DotCall {
            span: *span,
            kind: *kind,
            exp: exp_out,
            name: name.clone(),
            args: args_out,
            inferred_type: Some(typ_out_nf),
        })
    }
}
