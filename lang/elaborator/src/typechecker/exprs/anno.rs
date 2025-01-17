//! Bidirectional type checker

use crate::conversion_checking::convert;
use crate::normalizer::env::ToEnv;
use crate::normalizer::normalize::Normalize;
use ast::*;

use super::super::ctx::*;
use super::CheckInfer;
use crate::result::{TcResult, TypeError};

impl CheckInfer for Anno {
    fn check(&self, ctx: &mut Ctx, t: &Exp) -> TcResult<Self> {
        let inferred_term = self.infer(ctx)?;
        let inferred_typ = inferred_term.typ().ok_or(TypeError::Impossible {
            message: "Expected inferred type".to_owned(),
            span: None,
        })?;
        convert(ctx.vars.clone(), &mut ctx.meta_vars, inferred_typ, t, &self.span())?;
        Ok(inferred_term)
    }

    /// The *inference* rule for type annotations is:
    /// ```text
    ///            P, Γ ⊢ τ ⇐ Type
    ///            P, Γ ⊢ τ ▷ τ'
    ///            P, Γ ⊢ e ⇐ τ'
    ///           ──────────────────────
    ///            P, Γ ⊢ (e : τ) ⇒ τ'
    /// ```
    fn infer(&self, ctx: &mut Ctx) -> TcResult<Self> {
        let Anno { span, exp, typ, .. } = self;
        let typ_out = typ.check(ctx, &Box::new(TypeUniv::new().into()))?;
        let typ_nf = typ.normalize(&ctx.type_info_table, &mut ctx.env())?;
        let exp_out = (**exp).check(ctx, &typ_nf)?;
        Ok(Anno {
            span: *span,
            exp: Box::new(exp_out),
            typ: typ_out,
            normalized_type: Some(typ_nf),
        })
    }
}
