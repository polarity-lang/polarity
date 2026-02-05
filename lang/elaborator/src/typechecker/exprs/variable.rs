//! Bidirectional type checking for variables

use polarity_lang_ast::*;

use super::super::ctx::*;
use super::{CheckInfer, ExpectType};
use crate::conversion_checking::convert;
use crate::result::TcResult;
use crate::typechecker::erasure::is_runtime_irrelevant;

impl CheckInfer for Variable {
    /// The *checking* rule for variables is:
    /// ```text
    ///            P, Γ ⊢ x ⇒ τ
    ///            P, Γ ⊢ τ ≃ σ
    ///           ───────────────
    ///            P, Γ ⊢ x ⇐ σ
    /// ```
    fn check(&self, ctx: &mut Ctx, t: &Exp) -> TcResult<Self> {
        let inferred_term = self.infer(ctx)?;
        let inferred_typ = inferred_term.expect_typ()?;
        convert(&ctx.vars, &mut ctx.meta_vars, inferred_typ, t, &self.span())?;
        Ok(inferred_term)
    }

    /// The *inference* rule for variables is:
    /// ```text
    ///            Γ(x) = τ
    ///           ───────────────
    ///            P, Γ ⊢ x ⇒ τ
    /// ```
    fn infer(&self, ctx: &mut Ctx) -> TcResult<Self> {
        let Variable { span, idx, name, .. } = self;
        let typ_nf = ctx.lookup(*idx);
        let erased = is_runtime_irrelevant(&typ_nf);
        Ok(Variable {
            span: *span,
            idx: *idx,
            name: name.clone(),
            inferred_type: Some(typ_nf),
            erased,
        })
    }
}
