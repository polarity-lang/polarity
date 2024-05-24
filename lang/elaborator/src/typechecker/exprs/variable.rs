//! Bidirectional type checking for variables

use std::rc::Rc;

use askama::Template;
use log::trace;

use printer::PrintToString;

use syntax::ast::*;

use super::super::ctx::*;
use super::super::util::*;
use super::CheckInfer;
use crate::result::TypeError;

#[derive(Template)]
#[template(path = "var_rule.txt")]
struct VarRule<'a> {
    ctx: &'a str,
    var: &'a str,
    typ: &'a str,
}

impl CheckInfer for Variable {
    /// The *checking* rule for variables is:
    /// ```text
    ///            P, Γ ⊢ x ⇒ τ
    ///            P, Γ ⊢ τ ≃ σ
    ///           ───────────────
    ///            P, Γ ⊢ x ⇐ σ
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

    /// The *inference* rule for variables is:
    /// ```text
    ///            Γ(x) = τ
    ///           ───────────────
    ///            P, Γ ⊢ x ⇒ τ
    /// ```
    fn infer(&self, _prg: &Module, ctx: &mut Ctx) -> Result<Self, TypeError> {
        let Variable { span, idx, name, .. } = self;
        let typ_nf = ctx.lookup(*idx);

        trace!(
            "{}",
            VarRule {
                ctx: &ctx.vars.print_to_string(None),
                var: name,
                typ: &typ_nf.print_to_string(None)
            }
            .render()
            .unwrap()
        );

        Ok(Variable { span: *span, idx: *idx, name: name.clone(), inferred_type: Some(typ_nf) })
    }
}
