//! Bidirectional type checker

use std::rc::Rc;

use crate::normalizer::env::ToEnv;
use crate::normalizer::normalize::Normalize;
use syntax::ast::*;

use super::super::ctx::*;
use super::super::util::*;
use super::CheckInfer;
use crate::result::TypeError;

impl CheckInfer for Anno {
    fn check(&self, ctx: &mut Ctx, t: Rc<Exp>) -> Result<Self, TypeError> {
        let inferred_term = self.infer(ctx)?;
        let inferred_typ = inferred_term.typ().ok_or(TypeError::Impossible {
            message: "Expected inferred type".to_owned(),
            span: None,
        })?;
        convert(ctx.levels(), &mut ctx.meta_vars, inferred_typ, &t)?;
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
    fn infer(&self, ctx: &mut Ctx) -> Result<Self, TypeError> {
        let Anno { span, exp, typ, .. } = self;
        let typ_out = typ.check(ctx, Rc::new(TypeUniv::new().into()))?;
        let typ_nf = typ.normalize(&ctx.module, &mut ctx.env())?;
        let exp_out = (**exp).check(ctx, typ_nf.clone())?;
        Ok(Anno { span: *span, exp: Rc::new(exp_out), typ: typ_out, normalized_type: Some(typ_nf) })
    }
}
