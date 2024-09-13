//! Bidirectional type checker

use std::rc::Rc;

use syntax::ast::*;

use super::super::ctx::*;
use super::super::util::*;
use super::CheckInfer;
use crate::result::TypeError;

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
    fn check(&self, ctx: &mut Ctx, t: Rc<Exp>) -> Result<Self, TypeError> {
        convert(ctx.levels(), &mut ctx.meta_vars, Rc::new(TypeUniv::new().into()), &t)?;
        Ok(self.clone())
    }

    /// The *inference* rule for the type universe is:
    /// ```text
    ///           ─────────────────────
    ///            P, Γ ⊢ Type ⇒ Type
    /// ```
    /// Note: The type universe is impredicative and the theory
    /// therefore inconsistent.
    fn infer(&self, _ctx: &mut Ctx) -> Result<Self, TypeError> {
        Ok(self.clone())
    }
}
