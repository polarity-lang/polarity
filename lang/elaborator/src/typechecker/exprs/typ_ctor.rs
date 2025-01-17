//! Bidirectional type checking for type constructors

use ast::*;

use super::super::ctx::*;
use super::check_args;
use super::CheckInfer;
use crate::conversion_checking::convert;
use crate::result::TcResult;
use crate::result::TypeError;

impl CheckInfer for TypCtor {
    /// The *checking* rule for type constructors is:
    /// ```text
    ///            P, Γ ⊢ Tσ ⇒ ρ
    ///            P, Γ ⊢ τ ≃ ρ
    ///           ──────────────────
    ///            P, Γ ⊢ Tσ ⇐ τ
    /// ```
    fn check(&self, ctx: &mut Ctx, t: &Exp) -> TcResult<Self> {
        let inferred_term = self.infer(ctx)?;
        let inferred_typ = inferred_term.typ().ok_or(TypeError::Impossible {
            message: "Expected inferred type".to_owned(),
            span: None,
        })?;
        convert(ctx.vars.clone(), &mut ctx.meta_vars, inferred_typ, t, &self.span())?;
        Ok(inferred_term)
    }

    /// The *inference* rule for type constructors is:
    /// ```text
    ///            (co)data Tψ {...} ∈ P
    ///            P, Γ ⊢ σ ⇐ ψ
    ///           ─────────────────────────
    ///            P, Γ ⊢ Tσ ⇒ Type
    /// ```
    fn infer(&self, ctx: &mut Ctx) -> TcResult<Self> {
        let TypCtor { span, name, args } = self;
        let params = ctx.type_info_table.lookup_tyctor(name)?.params.clone();
        let args_out = check_args(args, name, ctx, &params, *span)?;

        Ok(TypCtor { span: *span, name: name.clone(), args: args_out })
    }
}
