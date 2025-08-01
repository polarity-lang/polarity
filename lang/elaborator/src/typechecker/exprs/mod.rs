pub mod anno;
pub mod call;
pub mod dot_call;
pub mod hole;
pub mod local_comatch;
pub mod local_let;
pub mod local_match;
pub mod typ_ctor;
pub mod type_univ;
pub mod variable;

use miette_util::ToMiette;
use miette_util::codespan::Span;

use log::trace;

use printer::types::Print;

use ast::ctx::LevelCtx;
use ast::*;

use super::ctx::*;
use crate::normalizer::{env::ToEnv, normalize::Normalize};
use crate::result::{TcResult, TypeError};

use ast::ctx::BindContext;
use ast::ctx::values::{Binder, Binding};

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
    /// - Γ: The context of locally bound variables
    /// - τ: The type we check against, must be in normal form.
    fn check(&self, ctx: &mut Ctx, t: &Exp) -> TcResult<Self>;
    /// Tries to infer a type for the given expression. For inference we use the
    /// following syntax:
    /// ```text
    ///            P, Γ ⊢ e ⇒ τ
    /// ```
    ///  - P: The program context of toplevel declarations.
    ///  - Γ: The context of locally bound variables.
    fn infer(&self, ctx: &mut Ctx) -> TcResult<Self>;
}

impl<T: CheckInfer> CheckInfer for Box<T> {
    fn check(&self, ctx: &mut Ctx, t: &Exp) -> TcResult<Self> {
        Ok(Box::new((**self).check(ctx, t)?))
    }
    fn infer(&self, ctx: &mut Ctx) -> TcResult<Self> {
        Ok(Box::new((**self).infer(ctx)?))
    }
}

trait ExpectType {
    fn expect_typ(&self) -> TcResult<Box<Exp>>;
}

impl<T: HasType> ExpectType for T {
    fn expect_typ(&self) -> TcResult<Box<Exp>> {
        self.typ().ok_or(Box::new(TypeError::Impossible {
            message: "Expected inferred type".to_owned(),
            span: None,
        }))
    }
}

// Expressions
//
//

impl CheckInfer for Exp {
    fn check(&self, ctx: &mut Ctx, t: &Exp) -> TcResult<Self> {
        trace!("{} |- {} <= {}", ctx.print_trace(), self.print_trace(), t.print_trace());
        match self {
            Exp::Variable(e) => Ok(e.check(ctx, t)?.into()),
            Exp::TypCtor(e) => Ok(e.check(ctx, t)?.into()),
            Exp::Call(e) => Ok(e.check(ctx, t)?.into()),
            Exp::DotCall(e) => Ok(e.check(ctx, t)?.into()),
            Exp::Anno(e) => Ok(e.check(ctx, t)?.into()),
            Exp::TypeUniv(e) => Ok(e.check(ctx, t)?.into()),
            Exp::Hole(e) => Ok(e.check(ctx, t)?.into()),
            Exp::LocalMatch(e) => Ok(e.check(ctx, t)?.into()),
            Exp::LocalComatch(e) => Ok(e.check(ctx, t)?.into()),
            Exp::LocalLet(e) => Ok(e.check(ctx, t)?.into()),
        }
    }

    fn infer(&self, ctx: &mut Ctx) -> TcResult<Self> {
        let res: TcResult<Exp> = match self {
            Exp::Variable(e) => Ok(e.infer(ctx)?.into()),
            Exp::TypCtor(e) => Ok(e.infer(ctx)?.into()),
            Exp::Call(e) => Ok(e.infer(ctx)?.into()),
            Exp::DotCall(e) => Ok(e.infer(ctx)?.into()),
            Exp::Anno(e) => Ok(e.infer(ctx)?.into()),
            Exp::TypeUniv(e) => Ok(e.infer(ctx)?.into()),
            Exp::Hole(e) => Ok(e.infer(ctx)?.into()),
            Exp::LocalMatch(e) => Ok(e.infer(ctx)?.into()),
            Exp::LocalComatch(e) => Ok(e.infer(ctx)?.into()),
            Exp::LocalLet(e) => Ok(e.infer(ctx)?.into()),
        };
        trace!(
            "{} |- {} => {}",
            ctx.print_trace(),
            self.print_trace(),
            res.as_ref().map(|e| e.typ()).print_trace()
        );
        res
    }
}

impl CheckInfer for Arg {
    fn check(&self, ctx: &mut Ctx, t: &Exp) -> TcResult<Self> {
        match self {
            Arg::UnnamedArg { arg, erased } => {
                Ok(Arg::UnnamedArg { arg: arg.check(ctx, t)?, erased: *erased })
            }
            Arg::NamedArg { name, arg, erased } => {
                Ok(Arg::NamedArg { name: name.clone(), arg: arg.check(ctx, t)?, erased: *erased })
            }
            Arg::InsertedImplicitArg { hole, erased } => {
                Ok(Arg::InsertedImplicitArg { hole: hole.check(ctx, t)?, erased: *erased })
            }
        }
    }

    fn infer(&self, ctx: &mut Ctx) -> TcResult<Self> {
        match self {
            Arg::UnnamedArg { arg, erased } => {
                Ok(Arg::UnnamedArg { arg: arg.infer(ctx)?, erased: *erased })
            }
            Arg::NamedArg { name, arg, erased } => {
                Ok(Arg::NamedArg { name: name.clone(), arg: arg.infer(ctx)?, erased: *erased })
            }
            Arg::InsertedImplicitArg { hole, erased } => {
                Ok(Arg::InsertedImplicitArg { hole: hole.infer(ctx)?, erased: *erased })
            }
        }
    }
}

fn check_args(
    this: &Args,
    name: &IdBound,
    ctx: &mut Ctx,
    params: &Telescope,
    span: Option<Span>,
) -> TcResult<Args> {
    if this.len() != params.len() {
        return Err(TypeError::ArgLenMismatch {
            name: name.to_owned().id,
            expected: params.len(),
            actual: this.len(),
            span: span.to_miette(),
        }
        .into());
    }

    let Telescope { params } =
        params.subst_in_telescope(LevelCtx::empty(), &vec![this.args.clone()])?;

    let args = this
        .args
        .iter()
        .zip(params)
        .map(|(exp, Param { typ, .. })| {
            let typ = typ.normalize(&ctx.type_info_table, &mut ctx.env())?;
            exp.check(ctx, &typ)
        })
        .collect::<Result<_, _>>()?;

    Ok(Args { args })
}

pub trait CheckTelescope {
    type Target;

    fn check_telescope<T, F: FnOnce(&mut Ctx, Self::Target) -> TcResult<T>>(
        &self,
        name: &str,
        ctx: &mut Ctx,
        params: &Telescope,
        f: F,
        span: Option<Span>,
    ) -> TcResult<T>;
}

pub trait InferTelescope {
    type Target;

    fn infer_telescope<T, F: FnOnce(&mut Ctx, Self::Target) -> TcResult<T>>(
        &self,
        ctx: &mut Ctx,
        f: F,
    ) -> TcResult<T>;
}

impl CheckTelescope for TelescopeInst {
    type Target = TelescopeInst;

    fn check_telescope<T, F: FnOnce(&mut Ctx, Self::Target) -> TcResult<T>>(
        &self,
        name: &str,
        ctx: &mut Ctx,
        param_types: &Telescope,
        f: F,
        span: Option<Span>,
    ) -> TcResult<T> {
        let Telescope { params: param_types } = param_types;
        let TelescopeInst { params } = self;

        if params.len() != param_types.len() {
            return Err(TypeError::ArgLenMismatch {
                name: name.to_owned(),
                expected: param_types.len(),
                actual: params.len(),
                span: span.to_miette(),
            }
            .into());
        }

        let iter = params.iter().zip(param_types);

        ctx.bind_fold_failable(
            iter,
            vec![],
            |ctx, params_out, (param_actual, param_expected)| {
                let ParamInst { span, name, .. } = param_actual;
                let Param { typ, erased, .. } = param_expected;
                let typ_out = typ.check(ctx, &Box::new(TypeUniv::new().into()))?;
                let typ_nf = typ.normalize(&ctx.type_info_table, &mut ctx.env())?;
                let param_out = ParamInst {
                    span: *span,
                    name: name.clone(),
                    typ: typ_out.into(),
                    erased: *erased,
                };
                params_out.push(param_out);
                let binder =
                    Binder { name: param_actual.name.clone(), content: Binding::from_type(typ_nf) };
                TcResult::<_>::Ok(binder)
            },
            |ctx, params| f(ctx, TelescopeInst { params }),
        )?
    }
}

impl InferTelescope for Telescope {
    type Target = Telescope;

    fn infer_telescope<T, F: FnOnce(&mut Ctx, Self::Target) -> TcResult<T>>(
        &self,
        ctx: &mut Ctx,
        f: F,
    ) -> TcResult<T> {
        let Telescope { params } = self;

        ctx.bind_fold_failable(
            params.iter(),
            vec![],
            |ctx, params_out, param| {
                let Param { implicit, typ, name, erased } = param;
                let typ_out = typ.check(ctx, &Box::new(TypeUniv::new().into()))?;
                let typ_nf = typ.normalize(&ctx.type_info_table, &mut ctx.env())?;
                let param_out = Param {
                    implicit: *implicit,
                    name: name.clone(),
                    typ: typ_out,
                    erased: *erased,
                };
                params_out.push(param_out);
                let binder =
                    Binder { name: param.name.clone(), content: Binding::from_type(typ_nf) };
                TcResult::<_>::Ok(binder)
            },
            |ctx, params| f(ctx, Telescope { params }),
        )?
    }
}

impl InferTelescope for SelfParam {
    type Target = SelfParam;

    fn infer_telescope<T, F: FnOnce(&mut Ctx, Self::Target) -> TcResult<T>>(
        &self,
        ctx: &mut Ctx,
        f: F,
    ) -> TcResult<T> {
        let SelfParam { info, name, typ } = self;

        let typ_nf = typ.to_exp().normalize(&ctx.type_info_table, &mut ctx.env())?;
        let typ_out = typ.infer(ctx)?;
        let param_out = SelfParam { info: *info, name: name.clone(), typ: typ_out };
        let elem = Binder { name: name.clone(), content: Binding::from_type(typ_nf) };

        // We need to shift the self parameter type here because we treat it as a 1-element telescope
        ctx.bind_single(shift_and_clone(&elem, (1, 0)), |ctx| f(ctx, param_out))
    }
}
