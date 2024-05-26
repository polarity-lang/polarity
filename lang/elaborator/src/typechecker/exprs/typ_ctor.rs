//! Bidirectional type checking for type constructors

use std::rc::Rc;

use syntax::ast::*;

use super::super::ctx::*;
use super::super::util::*;
use super::check_args;
use super::CheckInfer;
use crate::result::TypeError;

impl CheckInfer for TypCtor {
    /// The *checking* rule for type constructors is:
    /// ```text
    ///            P, Γ ⊢ Tσ ⇒ ρ
    ///            P, Γ ⊢ τ ≃ ρ
    ///           ──────────────────
    ///            P, Γ ⊢ Tσ ⇐ τ
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

    /// The *inference* rule for type constructors is:
    /// ```text
    ///            (co)data Tψ {...} ∈ P
    ///            P, Γ ⊢ σ ⇐ ψ
    ///           ─────────────────────────
    ///            P, Γ ⊢ Tσ ⇒ Type
    /// ```
    fn infer(&self, prg: &Module, ctx: &mut Ctx) -> Result<Self, TypeError> {
        let TypCtor { span, name, args } = self;
        let params = &*prg.typ(name, *span)?.typ();
        let args_out = check_args(args, prg, name, ctx, params, *span)?;

        Ok(TypCtor { span: *span, name: name.clone(), args: args_out })
    }
}
