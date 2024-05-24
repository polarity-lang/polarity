pub mod anno;
pub mod call;
pub mod dot_call;
pub mod hole;
pub mod local_comatch;
pub mod local_match;
pub mod typ_ctor;
pub mod type_univ;
pub mod typecheck;
pub mod variable;

use codespan::Span;
use miette_util::ToMiette;
pub use typecheck::*;

use std::rc::Rc;

use log::trace;

use printer::PrintToString;

use syntax::{ast::*, ctx::LevelCtx};

use super::ctx::*;
use crate::normalizer::{env::ToEnv, normalize::Normalize};
use crate::result::TypeError;

/// The CheckInfer trait for bidirectional type inference.
/// Expressions which implement this trait provide both a `check` function
/// to typecheck the expression against an expected type and a `infer` function
/// to infer the type of the given expression.
pub trait CheckInfer: Sized {
    /// Checks whether the expression has the given expected type. For checking we use
    /// the following syntax:
    /// ```text
    ///            P, Γ ⊢ e ⇐ τ
    /// ```
    /// - P: The program context of toplevel declarations.
    /// - Γ: The context of locally bound variables.
    fn check(&self, prg: &Module, ctx: &mut Ctx, t: Rc<Exp>) -> Result<Self, TypeError>;
    /// Tries to infer a type for the given expression. For inference we use the
    /// following syntax:
    /// ```text
    ///            P, Γ ⊢ e ⇒ τ
    /// ```
    ///  - P: The program context of toplevel declarations.
    ///  - Γ: The context of locally bound variables.
    fn infer(&self, prg: &Module, ctx: &mut Ctx) -> Result<Self, TypeError>;
}

impl<T: CheckInfer> CheckInfer for Rc<T> {
    fn check(&self, prg: &Module, ctx: &mut Ctx, t: Rc<Exp>) -> Result<Self, TypeError> {
        Ok(Rc::new((**self).check(prg, ctx, t)?))
    }
    fn infer(&self, prg: &Module, ctx: &mut Ctx) -> Result<Self, TypeError> {
        Ok(Rc::new((**self).infer(prg, ctx)?))
    }
}

// Expressions
//
//

impl CheckInfer for Exp {
    fn check(&self, prg: &Module, ctx: &mut Ctx, t: Rc<Exp>) -> Result<Self, TypeError> {
        trace!(
            "{} |- {} <= {}",
            ctx.print_to_string(None),
            self.print_to_string(None),
            t.print_to_string(None)
        );
        match self {
            Exp::Variable(e) => Ok(e.check(prg, ctx, t.clone())?.into()),
            Exp::TypCtor(e) => Ok(e.check(prg, ctx, t.clone())?.into()),
            Exp::Call(e) => Ok(e.check(prg, ctx, t.clone())?.into()),
            Exp::DotCall(e) => Ok(e.check(prg, ctx, t.clone())?.into()),
            Exp::Anno(e) => Ok(e.check(prg, ctx, t.clone())?.into()),
            Exp::TypeUniv(e) => Ok(e.check(prg, ctx, t.clone())?.into()),
            Exp::Hole(e) => Ok(e.check(prg, ctx, t.clone())?.into()),
            Exp::LocalMatch(e) => Ok(e.check(prg, ctx, t.clone())?.into()),
            Exp::LocalComatch(e) => Ok(e.check(prg, ctx, t.clone())?.into()),
        }
    }

    fn infer(&self, prg: &Module, ctx: &mut Ctx) -> Result<Self, TypeError> {
        let res: Result<Exp, TypeError> = match self {
            Exp::Variable(e) => Ok(e.infer(prg, ctx)?.into()),
            Exp::TypCtor(e) => Ok(e.infer(prg, ctx)?.into()),
            Exp::Call(e) => Ok(e.infer(prg, ctx)?.into()),
            Exp::DotCall(e) => Ok(e.infer(prg, ctx)?.into()),
            Exp::Anno(e) => Ok(e.infer(prg, ctx)?.into()),
            Exp::TypeUniv(e) => Ok(e.infer(prg, ctx)?.into()),
            Exp::Hole(e) => Ok(e.infer(prg, ctx)?.into()),
            Exp::LocalMatch(e) => Ok(e.infer(prg, ctx)?.into()),
            Exp::LocalComatch(e) => Ok(e.infer(prg, ctx)?.into()),
        };
        trace!(
            "{} |- {} => {}",
            ctx.print_to_string(None),
            self.print_to_string(None),
            res.as_ref().map(|e| e.typ()).print_to_string(None)
        );
        res
    }
}

fn check_args(
    this: &Args,
    prg: &Module,
    name: &str,
    ctx: &mut Ctx,
    params: &Telescope,
    span: Option<Span>,
) -> Result<Args, TypeError> {
    if this.len() != params.len() {
        return Err(TypeError::ArgLenMismatch {
            name: name.to_owned(),
            expected: params.len(),
            actual: this.len(),
            span: span.to_miette(),
        });
    }

    let Telescope { params } =
        params.subst_in_telescope(LevelCtx::empty(), &vec![this.args.clone()]);

    let args = this
        .args
        .iter()
        .zip(params)
        .map(|(exp, Param { typ, .. })| {
            let typ = typ.normalize(prg, &mut ctx.env())?;
            exp.check(prg, ctx, typ)
        })
        .collect::<Result<_, _>>()?;

    Ok(Args { args })
}
