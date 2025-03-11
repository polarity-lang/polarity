//! Bidirectional type checker

use ast::*;

use super::super::ctx::*;
use super::CheckInfer;
use crate::conversion_checking::convert;
use crate::result::TcResult;

// TypeUniv
//
//

impl CheckInfer for TypeUniv {
    /// The *checking* rule for the type universe is:
    /// ```text
    ///            P, Γ ⊢ τ ≃ Type
    ///           ──────────────────
    ///            P, Γ ⊢ Type ⇐ τ
    /// ```
    fn check(&self, ctx: &mut Ctx, t: &Exp) -> TcResult<Self> {
        convert(
            &ctx.vars,
            &mut ctx.meta_vars,
            Box::new(TypeUniv::new().into()),
            t,
            &self.span(),
        )?;
        Ok(self.clone())
    }

    /// The *inference* rule for the type universe is:
    /// ```text
    ///           ─────────────────────
    ///            P, Γ ⊢ Type ⇒ Type
    /// ```
    /// Note: The type universe is impredicative and the theory
    /// therefore inconsistent.
    fn infer(&self, _ctx: &mut Ctx) -> TcResult<Self> {
        Ok(self.clone())
    }
}
