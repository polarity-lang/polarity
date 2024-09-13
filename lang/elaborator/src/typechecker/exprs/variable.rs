//! Bidirectional type checking for variables

use std::rc::Rc;

use syntax::ast::*;

use super::super::ctx::*;
use super::super::util::*;
use super::CheckInfer;
use crate::result::TypeError;

impl CheckInfer for Variable {
    /// The *checking* rule for variables is:
    /// ```text
    ///            P, Γ ⊢ x ⇒ τ
    ///            P, Γ ⊢ τ ≃ σ
    ///           ───────────────
    ///            P, Γ ⊢ x ⇐ σ
    /// ```
    fn check(&self, ctx: &mut Ctx, t: Rc<Exp>) -> Result<Self, TypeError> {
        let inferred_term = self.infer(ctx)?;
        let inferred_typ = inferred_term.typ().ok_or(TypeError::Impossible {
            message: "Expected inferred type".to_owned(),
            span: None,
        })?;
        convert(ctx.levels(), &mut ctx.meta_vars, inferred_typ, &t)?;
        Ok(inferred_term)
    }

    /// The *inference* rule for variables is:
    /// ```text
    ///            Γ(x) = τ
    ///           ───────────────
    ///            P, Γ ⊢ x ⇒ τ
    /// ```
    fn infer(&self, ctx: &mut Ctx) -> Result<Self, TypeError> {
        let Variable { span, idx, name, .. } = self;
        let typ_nf = ctx.lookup(*idx);
        Ok(Variable { span: *span, idx: *idx, name: name.clone(), inferred_type: Some(typ_nf) })
    }
}
