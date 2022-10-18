//! Bidirectional type checker
//!
//! Notation:
//!
//! * `Δ` is the context of top-level declarations
//! * `Γ` is the context of local variables

use std::rc::Rc;

use data::HashSet;
use syntax::ast::{self, Substitutable};
use syntax::de_bruijn::*;
use syntax::elab::{self, ElabInfoExt};
use syntax::equiv::AlphaEq;
use syntax::named::Named;
use tracer::trace;

use super::ctx::*;
use super::result::TypeError;
use super::unify::*;

pub fn check(prg: &ast::Prg) -> Result<elab::Prg, TypeError> {
    let mut ctx = Ctx::build(prg);
    prg.infer(&mut ctx)
}

pub trait Infer {
    type Target;

    fn infer(&self, ctx: &mut Ctx) -> Result<Self::Target, TypeError>;
}

pub trait Check {
    type Target;

    fn check(&self, ctx: &mut Ctx, t: Rc<ast::Exp>) -> Result<Self::Target, TypeError>;
}

pub trait CheckArgs {
    type Target;

    fn check_args(&self, ctx: &mut Ctx, params: &ast::Telescope)
        -> Result<Self::Target, TypeError>;
}

pub trait CheckTelescope {
    type Target;

    fn check_telescope<T, F: FnOnce(&mut Ctx, Self::Target) -> Result<T, TypeError>>(
        &self,
        ctx: &mut Ctx,
        params: &ast::Telescope,
        f: F,
    ) -> Result<T, TypeError>;
}

pub trait InferTelescope {
    type Target;

    fn infer_telescope<T, F: FnOnce(&mut Ctx, Self::Target) -> Result<T, TypeError>>(
        &self,
        ctx: &mut Ctx,
        f: F,
    ) -> Result<T, TypeError>;
}

pub trait CheckEqns {
    type Target;

    fn check_eqns<T, F: FnOnce(&mut Ctx, Self::Target) -> Result<T, TypeError>>(
        &self,
        ctx: &mut Ctx,
        eqns: &[Eqn],
        f: F,
    ) -> Result<T, TypeError>;
}

pub trait Convert {
    fn convert(&self, other: &Self) -> Result<(), TypeError>;
}

struct WithDef<'a, T> {
    inner: &'a T,
    def: &'a ast::Def,
}

trait WithDefExt: Sized {
    fn with_def<'a>(&'a self, def: &'a ast::Def) -> WithDef<'a, Self>;
}

impl<T> WithDefExt for T {
    fn with_def<'a>(&'a self, def: &'a ast::Def) -> WithDef<'a, Self> {
        WithDef { inner: self, def }
    }
}

struct WithCodef<'a, T> {
    inner: &'a T,
    codef: &'a ast::Codef,
}

trait WithCodefExt: Sized {
    fn with_codef<'a>(&'a self, codef: &'a ast::Codef) -> WithCodef<'a, Self>;
}

impl<T> WithCodefExt for T {
    fn with_codef<'a>(&'a self, codef: &'a ast::Codef) -> WithCodef<'a, Self> {
        WithCodef { inner: self, codef }
    }
}

struct WithEqns<'a, T> {
    eqns: Vec<Eqn>,
    inner: &'a T,
}

trait WithEqnsExt: Sized {
    fn with_eqns(&self, eqns: Vec<Eqn>) -> WithEqns<'_, Self>;
}

impl<T> WithEqnsExt for T {
    fn with_eqns(&self, eqns: Vec<Eqn>) -> WithEqns<'_, Self> {
        WithEqns { eqns, inner: self }
    }
}

impl Infer for ast::Prg {
    type Target = elab::Prg;

    fn infer(&self, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let ast::Prg { decls, exp } = self;

        let decls_out = decls.infer(ctx)?;
        let exp_out = exp.as_ref().map(|exp| exp.infer(ctx)).transpose()?;

        Ok(elab::Prg { decls: decls_out, exp: exp_out })
    }
}

/// Infer all declarations in a program
///
/// ```text
/// ∀D ∊ Δ, D ⇒ ok
/// ―――――――――――――――
/// Δ ⇒ ok
/// ```
impl Infer for ast::Decls {
    type Target = elab::Decls;

    fn infer(&self, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let ast::Decls { map, order } = self;

        // FIXME: Reconsider order

        let map_out = map
            .iter()
            .map(|(name, decl)| Ok((name.clone(), decl.infer(ctx)?)))
            .collect::<Result<_, _>>()?;

        Ok(elab::Decls { map: map_out, order: order.clone() })
    }
}

/// Infer a declaration
impl Infer for ast::Decl {
    type Target = elab::Decl;

    #[trace("{} |- {} =>", ctx, self.name())]
    fn infer(&self, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let out = match self {
            ast::Decl::Data(data) => elab::Decl::Data(data.infer(ctx)?),
            ast::Decl::Codata(codata) => elab::Decl::Codata(codata.infer(ctx)?),
            ast::Decl::Ctor(ctor) => elab::Decl::Ctor(ctor.infer(ctx)?),
            ast::Decl::Dtor(dtor) => elab::Decl::Dtor(dtor.infer(ctx)?),
            ast::Decl::Def(def) => elab::Decl::Def(def.infer(ctx)?),
            ast::Decl::Codef(codef) => elab::Decl::Codef(codef.infer(ctx)?),
        };
        Ok(out)
    }
}

/// Infer a data declaration
///
/// ```text
/// (τ₀,…,τₙ): Type ⇒ ok
/// ――――――――――――――――――――――――――――――――
/// data D(τ₀,…,τₙ): Type := … ⇒ ok
/// ```
impl Infer for ast::Data {
    type Target = elab::Data;

    fn infer(&self, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let ast::Data { info, name, typ, ctors, impl_block } = self;

        let typ_out = typ.infer(ctx)?;

        Ok(elab::Data {
            info: info.clone().into(),
            name: name.clone(),
            typ: typ_out,
            ctors: ctors.clone(),
            impl_block: impl_block.clone().map(|block| block.into()),
        })
    }
}

/// Infer a codata declaration
///
/// ```text
/// (τ₀,…,τₙ): Type ⇒ ok
/// ―――――――――――――――――――――――――――――――――――
/// codata D(τ₀,…,τₙ): Type := … ⇒ ok
/// ```
impl Infer for ast::Codata {
    type Target = elab::Codata;

    fn infer(&self, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let ast::Codata { info, name, typ, dtors, impl_block } = self;

        let typ_out = typ.infer(ctx)?;

        Ok(elab::Codata {
            info: info.clone().into(),
            name: name.clone(),
            typ: typ_out,
            dtors: dtors.clone(),
            impl_block: impl_block.clone().map(|block| block.into()),
        })
    }
}

/// Infer a codata declaration
///
/// ```text
/// (τ₀,…,τₙ) ⇒ ok
/// ――――――――――――――――――――――
/// (τ₀,…,τₙ): Type ⇒ ok
/// ```
impl Infer for ast::TypAbs {
    type Target = elab::TypAbs;

    fn infer(&self, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let ast::TypAbs { params } = self;

        params.infer_telescope(ctx, |_, params_out| Ok(elab::TypAbs { params: params_out }))
    }
}

/// Infer a constructor declaration
///
/// ```text
/// Δ(C) = D
/// (α₀,…,αₙ) ⇒ ok
/// α₀,…,αₙ ⊢ D(β₀,…,βₘ) ⇒ ok
/// ――――――――――――――――――――――――――――
/// C(α₀,…,αₙ): D(β₀,…,βₘ) ⇒ ok
/// ```
impl Infer for ast::Ctor {
    type Target = elab::Ctor;

    fn infer(&self, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let ast::Ctor { info, name, params, typ } = self;

        // Check that the constructor lies in the data type it is defined in
        let expected = ctx.typ_for_xtor(name);
        if &typ.name != expected {
            return Err(TypeError::NotInType {
                expected: expected.clone(),
                actual: typ.name.clone(),
            });
        }

        params.infer_telescope(ctx, |ctx, params_out| {
            let typ_out = typ.infer(ctx)?;

            Ok(elab::Ctor {
                info: info.clone().into(),
                name: name.clone(),
                params: params_out,
                typ: typ_out,
            })
        })
    }
}

/// Infer a destructor declaration
///
/// ```text
/// Δ(d) = D
/// (α₀,…,αₙ) ⇒ ok
/// α₀,…,αₙ ⊢ D(β₀,…,βₘ) ⇒ ok
/// α₀,…,αₙ ⊢ τ ⇒ ok
/// ――――――――――――――――――――――――――――――
/// D(β₀,…,βₘ).d(α₀,…,αₙ): τ ⇒ ok
/// ```
impl Infer for ast::Dtor {
    type Target = elab::Dtor;

    fn infer(&self, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let ast::Dtor { info, name, params, on_typ, in_typ } = self;

        // Check that the destructor lies in the codata type it is defined in
        let expected = ctx.typ_for_xtor(name);
        if &on_typ.name != expected {
            return Err(TypeError::NotInType {
                expected: expected.clone(),
                actual: on_typ.name.clone(),
            });
        }

        params.infer_telescope(ctx, |ctx, params_out| {
            let on_typ_out = on_typ.infer(ctx)?;
            let in_typ_out = in_typ.infer(ctx)?;

            Ok(elab::Dtor {
                info: info.clone().into(),
                name: name.clone(),
                params: params_out,
                on_typ: on_typ_out,
                in_typ: in_typ_out,
            })
        })
    }
}

/// Infer a definition
///
/// ```text
/// (α₀,…,αₙ) ⇒ ok
/// α₀,…,αₙ ⊢ D(β₀,…,βₘ) ⇒ ok
/// α₀,…,αₙ ⊢ τ ⇒ ok
/// α₀,…,αₙ ⊢ M ⇐ τ
/// ―――――――――――――――――――――――――――――――――――――――
/// def D(β₀,…,βₘ).d(α₀,…,αₙ): τ := M ⇒ ok
/// ```
impl Infer for ast::Def {
    type Target = elab::Def;

    fn infer(&self, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let ast::Def { info, name, params, on_typ, in_typ, body } = self;

        params.infer_telescope(ctx, |ctx, params_out| {
            let on_typ_out = on_typ.infer(ctx)?;
            let in_typ_out = in_typ.infer(ctx)?;
            let body_out = body.with_def(self).check(ctx, in_typ.clone())?;
            Ok(elab::Def {
                info: info.clone().into(),
                name: name.clone(),
                params: params_out,
                on_typ: on_typ_out,
                in_typ: in_typ_out,
                body: body_out,
            })
        })
    }
}

/// Infer a co-definition
///
/// ```text
/// (α₀,…,αₙ) ⇒ ok
/// α₀,…,αₙ ⊢ D(β₀,…,βₘ) ⇒ ok
/// α₀,…,αₙ ⊢ M ⇒ ok
/// ―――――――――――――――――――――――――――――――――――――――
/// codef C(α₀,…,αₙ): D(β₀,…,βₘ) := M ⇒ ok
/// ```
impl Infer for ast::Codef {
    type Target = elab::Codef;

    fn infer(&self, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let ast::Codef { info, name, params, typ, body } = self;

        params.infer_telescope(ctx, |ctx, params_out| {
            let typ_out = typ.infer(ctx)?;
            let body_out = body.with_codef(self).infer(ctx)?;
            Ok(elab::Codef {
                info: info.clone().into(),
                name: name.clone(),
                params: params_out,
                typ: typ_out,
                body: body_out,
            })
        })
    }
}

/// Check a pattern match
///
/// ```text
/// Δ ⊢ c₀,…,cₙ valid for D
/// ∀i,
///     cᵢ = C(α₀,…,αₙ) => e
///     Δ ⊢ C(β₀,…,βₘ): D(γ₀,…,γᵤ)
///     E = { δⱼ = γⱼ | ∀j }
///     E ⊢ cᵢ ⇐ τ
/// ―――――――――――――――――――――――――――――――――――
/// D(δ₀,…,δᵥ).d ⊢ match c₀,…,cₙ ⇐ τ
/// ```
impl<'a> Check for WithDef<'a, ast::Match> {
    type Target = elab::Match;

    fn check(&self, ctx: &mut Ctx, t: Rc<ast::Exp>) -> Result<Self::Target, TypeError> {
        let ast::Match { info, cases } = &self.inner;

        // Check exhaustiveness
        let ctors_expected = ctx.xtors_for_typ(ctx.typ_for_xtor(&self.def.name));
        let mut ctors_actual = HashSet::default();
        let mut ctors_duplicate = HashSet::default();

        for name in cases.iter().map(|case| &case.name) {
            if ctors_actual.contains(name) {
                ctors_duplicate.insert(name.clone());
            }
            ctors_actual.insert(name.clone());
        }
        let mut ctors_missing = ctors_expected.difference(&ctors_actual).peekable();
        let mut ctors_undeclared = ctors_actual.difference(ctors_expected).peekable();

        if ctors_missing.peek().is_some()
            || ctors_undeclared.peek().is_some()
            || !ctors_duplicate.is_empty()
        {
            return Err(TypeError::InvalidMatch {
                missing: ctors_missing.cloned().collect(),
                undeclared: ctors_undeclared.cloned().collect(),
                duplicate: ctors_duplicate,
            });
        }

        // Consider all cases
        let cases_out: Vec<_> = cases
            .iter()
            .map(|case| {
                // Build equations for this case
                let ast::Ctor { typ: ast::TypApp { args: def_args, .. }, .. } =
                    &*ctx.ctor(&case.name);
                let ast::TypApp { args: on_args, .. } = &self.def.on_typ;
                let on_args = on_args.shift((1, 0)); // FIXME: where to shift this

                let eqns: Vec<_> =
                    on_args.iter().cloned().zip(def_args.iter().cloned()).map(Eqn::from).collect();

                // Check the case given the equations
                case.with_eqns(eqns).check(ctx, t.clone())
            })
            .collect::<Result<_, _>>()?;

        Ok(elab::Match { info: info.clone().into(), cases: cases_out })
    }
}

/// Infer a copattern match
///
/// ```text
/// Δ ⊢ d₀,…,dₙ valid for D
/// ∀i,
///     dᵢ = d(α₀,…,αₙ) => e
///     Δ ⊢ D(γ₀,…,γᵤ).d(β₀,…,βₘ): τ
///     E = { δⱼ = γⱼ | ∀j }
///     E ⊢ dᵢ ⇐ τ
/// ―――――――――――――――――――――――――――――――――――
/// C(δ₀,…,δᵥ): D(ε₀,…,εₛ) ⊢ comatch c₀,…,cₙ ⇒ ok
/// ```
impl<'a> Infer for WithCodef<'a, ast::Comatch> {
    type Target = elab::Comatch;

    fn infer(&self, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let ast::Comatch { info, cases } = &self.inner;

        // Check exhaustiveness
        let dtors_expected = ctx.xtors_for_typ(ctx.typ_for_xtor(&self.codef.name));
        let mut dtors_actual = HashSet::default();
        let mut dtors_duplicate = HashSet::default();

        for name in cases.iter().map(|case| &case.name) {
            if dtors_actual.contains(name) {
                dtors_duplicate.insert(name.clone());
            }
            dtors_actual.insert(name.clone());
        }

        let mut dtors_missing = dtors_expected.difference(&dtors_actual).peekable();
        let mut dtors_exessive = dtors_actual.difference(dtors_expected).peekable();

        if dtors_missing.peek().is_some()
            || dtors_exessive.peek().is_some()
            || !dtors_duplicate.is_empty()
        {
            return Err(TypeError::InvalidMatch {
                missing: dtors_missing.cloned().collect(),
                undeclared: dtors_exessive.cloned().collect(),
                duplicate: dtors_duplicate,
            });
        }

        // Consider all cases
        let cases_out: Vec<_> = cases
            .iter()
            .map(|case| {
                // Build equations for this case
                let ast::Dtor { on_typ: ast::TypApp { args: def_args, .. }, in_typ, .. } =
                    &*ctx.dtor(&case.name);

                let ast::TypApp { args: on_args, .. } = &self.codef.typ;
                let on_args = on_args.shift((1, 0)); // FIXME: where to shift this

                let eqns: Vec<_> =
                    on_args.iter().cloned().zip(def_args.iter().cloned()).map(Eqn::from).collect();

                // Check the case given the equations
                case.with_eqns(eqns).check(ctx, in_typ.clone())
            })
            .collect::<Result<_, _>>()?;

        Ok(elab::Comatch { info: info.clone().into(), cases: cases_out })
    }
}

/// Infer a case in a pattern match
///
/// ```text
/// Δ ⊢ C(β₀,…,βₘ): D(γ₀,…,γᵤ)
/// Γ ⊢ (α₀,…,αₙ) = (β₀,…,βₘ)
/// Γ,[α₀,…,αₙ] ⊢ h₀,…,hᵥ ⇐ E
/// Γ' = Γ,[α₀,…,αₙ],[h₀,…,hᵥ]
/// Γ' ⊢ e ⇒ τ₁
/// Γ' ⊢ unify E ⤳ σ
/// Γ' ⊢ τ₀[σ] = τ₁[σ]
/// ―――――――――――――――――――――――――――――――――――
/// E | Γ ⊢ C(α₀,…,αₙ){h₀,…,hᵥ} => e ⇐ τ₀
/// ```
impl<'a> Check for WithEqns<'a, ast::Case> {
    type Target = elab::Case;

    #[trace("{} |- {:P} <= {:P}", ctx, self.inner, t)]
    fn check(&self, ctx: &mut Ctx, t: Rc<ast::Exp>) -> Result<Self::Target, TypeError> {
        let ast::Case { info, name, args, body } = self.inner;
        let ast::Ctor { name, params, .. } = &*ctx.ctor(name);

        args.check_telescope(ctx, params, |ctx, args_out| {
            let body_out = match body {
                Some(body) => {
                    let unif = unify(ctx, self.eqns.clone())
                        .map_err(TypeError::Unify)?
                        .map_no(|()| TypeError::PatternIsAbsurd { name: name.clone() })
                        .ok_yes()?;

                    // FIXME: Track substitution in context instead
                    let mut ctx = ctx.subst(ctx, &unif);
                    let body = body.subst(&ctx, &unif);
                    let ctx = &mut ctx;

                    let body_out = body.infer(ctx)?;

                    body_out.typ().convert(&t.shift((1, 0)).subst(ctx, &unif))?;

                    Some(body_out)
                }
                None => {
                    unify(ctx, self.eqns.clone())
                        .map_err(TypeError::Unify)?
                        .map_yes(|_| TypeError::PatternIsNotAbsurd { name: name.clone() })
                        .ok_no()?;

                    None
                }
            };

            Ok(elab::Case {
                info: info.clone().into(),
                name: name.clone(),
                args: args_out,
                body: body_out,
            })
        })
    }
}

/// Infer a cocase in a co-pattern match
///
/// ```text
/// Δ ⊢ D(γ₀,…,γᵤ).d(β₀,…,βₘ): τ₀
/// Γ ⊢ (α₀,…,αₙ) = (β₀,…,βₘ)
/// Γ,[α₀,…,αₙ] ⊢ h₀,…,hᵥ ⇐ E
/// Γ' = Γ,[α₀,…,αₙ],[h₀,…,hᵥ]
/// Γ' ⊢ e ⇒ τ₁
/// Γ' ⊢ unify E ⤳ σ
/// Γ' ⊢ τ₀[σ] = τ₁[σ]
/// ―――――――――――――――――――――――――――――――――――
/// E | Γ ⊢ d(α₀,…,αₙ){h₀,…,hᵥ} => e ⇐ τ₀
/// ```
impl<'a> Check for WithEqns<'a, ast::Cocase> {
    type Target = elab::Cocase;

    #[trace("{} |- {:P} <= {:P}", ctx, self.inner, t)]
    fn check(&self, ctx: &mut Ctx, t: Rc<ast::Exp>) -> Result<Self::Target, TypeError> {
        let ast::Cocase { info, name, args, body } = self.inner;
        let ast::Dtor { name, params, .. } = &*ctx.dtor(name);

        args.check_telescope(ctx, params, |ctx, args_out| {
            let body_out = match body {
                Some(body) => {
                    let unif = unify(ctx, self.eqns.clone())
                        .map_err(TypeError::Unify)?
                        .map_no(|()| TypeError::PatternIsAbsurd { name: name.clone() })
                        .ok_yes()?;

                    // FIXME: Track substitution in context instead
                    let mut ctx = ctx.subst(ctx, &unif);
                    let body = body.subst(&ctx, &unif);
                    let ctx = &mut ctx;

                    let body_out = body.infer(ctx)?;

                    body_out.typ().convert(&t.shift((0, 0)).subst(ctx, &unif))?;

                    Some(body_out)
                }
                None => {
                    unify(ctx, self.eqns.clone())
                        .map_err(TypeError::Unify)?
                        .map_yes(|_| TypeError::PatternIsNotAbsurd { name: name.clone() })
                        .ok_no()?;

                    None
                }
            };

            Ok(elab::Cocase {
                info: info.clone().into(),
                name: name.clone(),
                args: args_out,
                body: body_out,
            })
        })
    }
}

/// Check an expression
///
/// ```text
/// Γ ⊢ e ⇒ τ₁
/// Γ ⊢ τ₀ = τ₁
/// ―――――――――――――――――――――――――――――――――――
/// Γ ⊢ e ⇐ τ₀
/// ```
impl Check for ast::Exp {
    type Target = elab::Exp;

    #[trace("{} |- {:P} <= {:P}", ctx, self, t)]
    fn check(&self, ctx: &mut Ctx, t: Rc<ast::Exp>) -> Result<Self::Target, TypeError> {
        let actual = self.infer(ctx)?;
        actual.typ().convert(&t)?;
        Ok(actual)
    }
}

/// Infer an expression
///
/// ```text
/// Γ(x) = τ
/// ―――――――――― (Var)
/// Γ ⊢ x ⇒ τ
/// ```
///
/// ```text
/// Δ ⊢ D(β₀,…,βₙ) : Type
/// Γ ⊢ (α₀,…,αₙ) ⇐ β₀,…,βₙ
/// ―――――――――――――――――――――――― (TypCtor)
/// Γ ⊢ D(α₀,…,αₙ) ⇒ Type
/// ```
///
/// ```text
/// Δ ⊢ C(γ₀,…,γₙ) : D(β₀,…,βₘ)
/// Γ ⊢ (α₀,…,αₙ) ⇐ γ₀,…,γₙ
/// ――――――――――――――――――――――――――――――――――――――― (Ctor)
/// Γ ⊢ C(α₀,…,αₙ) ⇒ D((β₀,…,βₘ)[α₀,…,αₙ])
/// ```
///
/// ```text
/// Δ ⊢ D(β₀,…,βₘ).d(γ₀,…,γₙ) : τ
/// Γ ⊢ (α₀,…,αₙ) ⇐ (γ₀,…,γₙ)
/// Γ ⊢ e ⇐ D((β₀,…,βₘ)[α₀,…,αₙ])
/// ―――――――――――――――――――――――――――――― (Dtor)
/// Γ ⊢ e.d(α₀,…,αₙ) ⇒ τ[α₀,…,αₙ]
/// ```
///
/// ```text
/// Γ ⊢ τ : Type
/// Γ ⊢ e ⇐ τ
/// ―――――――――――――― (Anno)
/// Γ ⊢ e : τ ⇒ τ
/// ```
///
/// ```text
/// ―――――――――――――― (Type)
/// Γ ⊢ Type : Type
/// ```
impl Infer for ast::Exp {
    type Target = elab::Exp;

    #[trace("{} |- {:P} => {return:P}", ctx, self, |ret| ret.as_ref().map(|e| e.typ()))]
    fn infer(&self, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        match self {
            ast::Exp::Var { info, name, idx } => {
                let typ = ctx.bound(*idx);
                Ok(elab::Exp::Var { info: info.with_type(typ), name: name.clone(), idx: *idx })
            }
            ast::Exp::TypCtor { info, name, args } => {
                let ast::TypAbs { params } = &*ctx.typ(name);

                let args_out = args.check_args(ctx, params)?;

                Ok(elab::Exp::TypCtor {
                    info: info.with_type(Rc::new(ast::Exp::Type { info: ast::Info::empty() })),
                    name: name.clone(),
                    args: args_out,
                })
            }
            ast::Exp::Ctor { info, name, args } => {
                let ast::Ctor { name, params, typ, .. } = &*ctx.ctor(name);

                let args_out = args.check_args(ctx, params)?;
                let typ_out = typ.subst_under_telescope(params, args).to_exp();

                Ok(elab::Exp::Ctor {
                    info: info.with_type(Rc::new(typ_out)),
                    name: name.clone(),
                    args: args_out,
                })
            }
            ast::Exp::Dtor { info, exp, name, args } => {
                let ast::Dtor { name, params, on_typ, in_typ, .. } = &*ctx.dtor(name);

                let args_out = args.check_args(ctx, params)?;

                let on_typ_out = on_typ.subst_under_telescope(params, args);
                let typ_out = in_typ.subst_under_telescope(params, args);

                let exp_out = exp.check(ctx, Rc::new(on_typ_out.to_exp()))?;

                Ok(elab::Exp::Dtor {
                    info: info.with_type(typ_out),
                    exp: exp_out,
                    name: name.clone(),
                    args: args_out,
                })
            }
            ast::Exp::Anno { info, exp, typ } => {
                let typ_out =
                    typ.check(ctx, Rc::new(ast::Exp::Type { info: ast::Info::empty() }))?;
                let exp_out = (**exp).check(ctx, typ.clone())?;
                Ok(elab::Exp::Anno {
                    info: info.with_type(typ.clone()),
                    exp: Rc::new(exp_out),
                    typ: typ_out,
                })
            }
            ast::Exp::Type { info } => Ok(elab::Exp::Type {
                info: info.with_type(Rc::new(ast::Exp::Type { info: ast::Info::empty() })),
            }),
        }
    }
}

/// ```text
/// Δ ⊢ D(β₀,…,βₙ) : Type
/// Γ ⊢ (α₀,…,αₙ) ⇐ (β₀,…,βₙ)
/// ――――――――――――――――――――――――――
/// Γ ⊢ D(α₀,…,αₙ) ⇒ ok
/// ```
impl Infer for ast::TypApp {
    type Target = elab::TypApp;

    fn infer(&self, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let ast::TypApp { info, name, args } = self;
        let ast::TypAbs { params } = &*ctx.typ(name);

        let args_out = args.check_args(ctx, params)?;
        Ok(elab::TypApp {
            info: info.with_type(Rc::new(ast::Exp::Type { info: ast::Info::empty() })),
            name: name.clone(),
            args: args_out,
        })
    }
}

/// ```text
/// ∀i,
///     Γ ⊢ αᵢ ⇐ βᵢ[α₀,…,αₙ]
/// ――――――――――――――――――――――――――
/// Γ ⊢ (α₀,…,αₙ) ⇐ (β₀,…,βₙ)
/// ```
impl CheckArgs for ast::Args {
    type Target = elab::Args;

    fn check_args(
        &self,
        ctx: &mut Ctx,
        params: &ast::Telescope,
    ) -> Result<Self::Target, TypeError> {
        if self.len() != params.len() {
            return Err(TypeError::ArgLenMismatch { expected: params.len(), actual: self.len() });
        }

        let ast::Telescope { params } = params.subst_in_telescope(self);

        self.iter().zip(params).map(|(exp, ast::Param { typ, .. })| exp.check(ctx, typ)).collect()
    }
}

/// ```text
/// ∀i,
///     Γ,[a₀,…,aᵢ₋₁] ⊢ aᵢ ⇐ Type
///     Γ,[a₀,…,aᵢ₋₁] ⊢ aᵢ = βᵢ
/// ――――――――――――――――――――――――――――
/// Γ ⊢ (α₀,…,αₙ) = (β₀,…,βₘ)
/// ```
impl CheckTelescope for ast::Telescope {
    type Target = elab::Telescope;

    fn check_telescope<T, F: FnOnce(&mut Ctx, Self::Target) -> Result<T, TypeError>>(
        &self,
        ctx: &mut Ctx,
        param_types: &ast::Telescope,
        f: F,
    ) -> Result<T, TypeError> {
        let ast::Telescope { params: param_types } = param_types;
        let ast::Telescope { params } = self;

        if params.len() != param_types.len() {
            return Err(TypeError::ArgLenMismatch {
                expected: param_types.len(),
                actual: params.len(),
            });
        }

        let iter = params.iter().zip(param_types);

        ctx.bind_fold(
            iter,
            Ok(elab::Params::new()),
            |ctx, params_out, (param_actual, param_expected)| {
                let ast::Param { typ: typ_actual, name } = param_actual;
                let ast::Param { typ: typ_expected, .. } = param_expected;
                let mut params_out = params_out?;
                typ_actual.convert(typ_expected)?;
                let typ_out =
                    typ_actual.check(ctx, Rc::new(ast::Exp::Type { info: ast::Info::empty() }))?;
                let param_out = elab::Param { name: name.clone(), typ: typ_out };
                params_out.push(param_out);
                Ok(params_out)
            },
            |ctx, params| f(ctx, elab::Telescope { params: params? }),
        )
    }
}

/// ```text
/// ∀i,
///     Γ,[a₀,…,aᵢ₋₁] ⊢ aᵢ ⇒ ok
/// ――――――――――――――――――――――――――――
/// Γ ⊢ (α₀,…,αₙ) ⇒ ok
/// ```
impl InferTelescope for ast::Telescope {
    type Target = elab::Telescope;

    fn infer_telescope<T, F: FnOnce(&mut Ctx, Self::Target) -> Result<T, TypeError>>(
        &self,
        ctx: &mut Ctx,
        f: F,
    ) -> Result<T, TypeError> {
        let ast::Telescope { params } = self;

        ctx.bind_fold(
            params.iter(),
            Ok(elab::Params::new()),
            |ctx, params_out, param| {
                let ast::Param { typ, name } = param;
                let mut params_out = params_out?;
                let typ_out = typ.infer(ctx)?;
                let param_out = elab::Param { name: name.clone(), typ: typ_out };
                params_out.push(param_out);
                Ok(params_out)
            },
            |ctx, params| f(ctx, elab::Telescope { params: params? }),
        )
    }
}

trait SubstUnderTelescope {
    /// Substitute under a telescope
    fn subst_under_telescope(&self, telescope: &ast::Telescope, args: &[Rc<ast::Exp>]) -> Self;
}

impl<T: Substitutable + Clone> SubstUnderTelescope for T {
    fn subst_under_telescope(&self, telescope: &ast::Telescope, args: &[Rc<ast::Exp>]) -> Self {
        let ast::Telescope { params } = telescope;

        let in_exp = self.clone();

        Ctx::empty().bind_fold(params.iter(), (), |_, _, _| (), |ctx, _| in_exp.subst(ctx, &args))
    }
}

trait SubstInTelescope {
    /// Substitute in a telescope
    fn subst_in_telescope(&self, args: &[Rc<ast::Exp>]) -> Self;
}

impl SubstInTelescope for ast::Telescope {
    fn subst_in_telescope(&self, args: &[Rc<ast::Exp>]) -> Self {
        let ast::Telescope { params } = self;

        Ctx::empty().bind_fold(
            params.iter(),
            Vec::new(),
            |ctx, mut params_out, param| {
                params_out.push(param.subst(ctx, &args));
                params_out
            },
            |_, params_out| ast::Telescope { params: params_out },
        )
    }
}

impl<T1, T2: Typed> Typed for (T1, T2) {
    fn typ(&self) -> Rc<ast::Exp> {
        self.1.typ()
    }
}

impl Typed for &syntax::ast::Param {
    fn typ(&self) -> Rc<ast::Exp> {
        self.typ.clone()
    }
}

impl Typed for elab::Exp {
    fn typ(&self) -> Rc<ast::Exp> {
        match self {
            elab::Exp::Var { info, .. } => info.typ.clone(),
            elab::Exp::TypCtor { info, .. } => info.typ.clone(),
            elab::Exp::Ctor { info, .. } => info.typ.clone(),
            elab::Exp::Dtor { info, .. } => info.typ.clone(),
            elab::Exp::Anno { info, .. } => info.typ.clone(),
            elab::Exp::Type { info } => info.typ.clone(),
        }
    }
}

impl Convert for Rc<ast::Exp> {
    #[trace("{:P} =? {:P}", self, other)]
    fn convert(&self, other: &Self) -> Result<(), TypeError> {
        self.alpha_eq(other)
            .then_some(())
            .ok_or_else(|| TypeError::NotEq { lhs: self.clone(), rhs: other.clone() })
    }
}

impl Convert for Eqn {
    fn convert(&self, other: &Self) -> Result<(), TypeError> {
        self.lhs.convert(&other.lhs)?;
        self.rhs.convert(&other.rhs)
    }
}

impl<T: Check> Check for Rc<T> {
    type Target = Rc<T::Target>;

    fn check(&self, ctx: &mut Ctx, t: Rc<ast::Exp>) -> Result<Self::Target, TypeError> {
        Ok(Rc::new((**self).check(ctx, t)?))
    }
}

impl<T: Infer> Infer for Rc<T> {
    type Target = Rc<T::Target>;

    fn infer(&self, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        Ok(Rc::new((**self).infer(ctx)?))
    }
}
