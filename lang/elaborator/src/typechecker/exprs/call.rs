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

impl CheckInfer for Call {
    /// The *checking* rule for calls is:
    /// ```text
    ///                 ...
    ///           ──────────────────
    ///            P, Γ ⊢ Cσ ⇐ τ
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
    /// The *inference* rule for calls is:
    /// ```text
    ///                 ...
    ///           ──────────────────
    ///            P, Γ ⊢ Cσ ⇒ ...
    /// ```
    fn infer(&self, prg: &Module, ctx: &mut Ctx) -> Result<Self, TypeError> {
        let Call { span, kind, name, args, .. } = self;

        match kind {
            CallKind::Codefinition | CallKind::Constructor => {
                let Ctor { name, params, typ, .. } = &prg.ctor_or_codef(name, *span)?;
                let args_out = check_args(args, prg, name, ctx, params, *span)?;
                let typ_out = typ
                    .subst_under_ctx(vec![params.len()].into(), &vec![args.args.clone()])
                    .to_exp();
                let typ_nf = typ_out.normalize(prg, &mut ctx.env())?;
                Ok(Call {
                    span: *span,
                    kind: *kind,
                    name: name.clone(),
                    args: args_out,
                    inferred_type: Some(typ_nf),
                })
            }
            CallKind::LetBound => {
                let Let { name, params, typ, .. } = prg.top_level_let(name, *span)?;
                let args_out = check_args(args, prg, name, ctx, params, *span)?;
                let typ_out =
                    typ.subst_under_ctx(vec![params.len()].into(), &vec![args.args.clone()]);
                let typ_nf = typ_out.normalize(prg, &mut ctx.env())?;
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
