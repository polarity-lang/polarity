//! Bidirectional type checker

use std::rc::Rc;

use data::HashSet;
use miette_util::ToMiette;
use syntax::ast::forget::Forget;
use syntax::ast::Substitutable;
use syntax::common::HasInfo;
use syntax::ctx::{Bind, Context};
use syntax::de_bruijn::*;
use syntax::equiv::AlphaEq;
use syntax::named::Named;
use syntax::tst::{self, ElabInfoExt, HasTypeInfo};
use syntax::ust;
use tracer::trace;

use super::ctx::*;
use super::prg::*;
use super::result::TypeError;
use super::unify::*;

pub fn check(prg: &ust::Prg) -> Result<tst::Prg, TypeError> {
    let prg_ctx = Prg::build(prg);
    let mut var_ctx = Ctx::empty();
    prg.infer(&prg_ctx, &mut var_ctx)
}

pub trait Infer {
    type Target;

    fn infer(&self, prg: &Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError>;
}

pub trait Check {
    type Target;

    fn check(&self, prg: &Prg, ctx: &mut Ctx, t: Rc<ust::Exp>) -> Result<Self::Target, TypeError>;
}

pub trait CheckArgs {
    type Target;

    fn check_args(
        &self,
        prg: &Prg,
        name: &String,
        ctx: &mut Ctx,
        params: &ust::Telescope,
    ) -> Result<Self::Target, TypeError>;
}

pub trait CheckTelescope {
    type Target;

    fn check_telescope<T, F: FnOnce(&mut Ctx, Self::Target) -> Result<T, TypeError>>(
        &self,
        prg: &Prg,
        name: &String,
        ctx: &mut Ctx,
        params: &ust::Telescope,
        f: F,
    ) -> Result<T, TypeError>;
}

pub trait InferTelescope {
    type Target;

    fn infer_telescope<T, F: FnOnce(&mut Ctx, Self::Target) -> Result<T, TypeError>>(
        &self,
        prg: &Prg,
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

struct WithScrutinee<'a, T> {
    inner: &'a T,
    scrutinee: ust::TypApp,
}

trait WithScrutineeExt: Sized {
    fn with_scrutinee(&self, scrutinee: ust::TypApp) -> WithScrutinee<'_, Self>;
}

impl<T> WithScrutineeExt for T {
    fn with_scrutinee(&self, scrutinee: ust::TypApp) -> WithScrutinee<'_, Self> {
        WithScrutinee { inner: self, scrutinee }
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

impl Infer for ust::Prg {
    type Target = tst::Prg;

    fn infer(&self, prg: &Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let ust::Prg { decls, exp } = self;

        let decls_out = decls.infer(prg, ctx)?;
        let exp_out = exp.as_ref().map(|exp| exp.infer(prg, ctx)).transpose()?;

        Ok(tst::Prg { decls: decls_out, exp: exp_out })
    }
}

/// Infer all declarations in a program
impl Infer for ust::Decls {
    type Target = tst::Decls;

    fn infer(&self, prg: &Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let ust::Decls { map, source } = self;

        // FIXME: Reconsider order

        let map_out = map
            .iter()
            .map(|(name, decl)| Ok((name.clone(), decl.infer(prg, ctx)?)))
            .collect::<Result<_, TypeError>>()?;

        Ok(tst::Decls { map: map_out, source: source.clone() })
    }
}

/// Infer a declaration
impl Infer for ust::Decl {
    type Target = tst::Decl;

    #[trace("{:P} |- {} =>", ctx, self.name())]
    fn infer(&self, prg: &Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let out = match self {
            ust::Decl::Data(data) => tst::Decl::Data(data.infer(prg, ctx)?),
            ust::Decl::Codata(codata) => tst::Decl::Codata(codata.infer(prg, ctx)?),
            ust::Decl::Ctor(ctor) => tst::Decl::Ctor(ctor.infer(prg, ctx)?),
            ust::Decl::Dtor(dtor) => tst::Decl::Dtor(dtor.infer(prg, ctx)?),
            ust::Decl::Def(def) => tst::Decl::Def(def.infer(prg, ctx)?),
            ust::Decl::Codef(codef) => tst::Decl::Codef(codef.infer(prg, ctx)?),
        };
        Ok(out)
    }
}

/// Infer a data declaration
impl Infer for ust::Data {
    type Target = tst::Data;

    fn infer(&self, prg: &Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let ust::Data { info, name, typ, ctors, impl_block } = self;

        let typ_out = typ.infer(prg, ctx)?;

        Ok(tst::Data {
            info: info.clone().into(),
            name: name.clone(),
            typ: typ_out,
            ctors: ctors.clone(),
            impl_block: impl_block.clone().map(|block| block.into()),
        })
    }
}

/// Infer a codata declaration
impl Infer for ust::Codata {
    type Target = tst::Codata;

    fn infer(&self, prg: &Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let ust::Codata { info, name, typ, dtors, impl_block } = self;

        let typ_out = typ.infer(prg, ctx)?;

        Ok(tst::Codata {
            info: info.clone().into(),
            name: name.clone(),
            typ: typ_out,
            dtors: dtors.clone(),
            impl_block: impl_block.clone().map(|block| block.into()),
        })
    }
}

/// Infer a codata declaration
impl Infer for ust::TypAbs {
    type Target = tst::TypAbs;

    fn infer(&self, prg: &Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let ust::TypAbs { params } = self;

        params.infer_telescope(prg, ctx, |_, params_out| Ok(tst::TypAbs { params: params_out }))
    }
}

/// Infer a constructor declaration
impl Infer for ust::Ctor {
    type Target = tst::Ctor;

    fn infer(&self, prg: &Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let ust::Ctor { info, name, params, typ } = self;

        // Check that the constructor lies in the data type it is defined in
        let expected = prg.typ_for_xtor(name);
        if &typ.name != expected {
            return Err(TypeError::NotInType {
                expected: expected.clone(),
                actual: typ.name.clone(),
                span: typ.info.span.to_miette(),
            });
        }

        params.infer_telescope(prg, ctx, |ctx, params_out| {
            let typ_out = typ.infer(prg, ctx)?;

            Ok(tst::Ctor {
                info: info.clone().into(),
                name: name.clone(),
                params: params_out,
                typ: typ_out,
            })
        })
    }
}

/// Infer a destructor declaration
impl Infer for ust::Dtor {
    type Target = tst::Dtor;

    fn infer(&self, prg: &Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let ust::Dtor { info, name, params, on_typ, in_typ } = self;

        // Check that the destructor lies in the codata type it is defined in
        let expected = prg.typ_for_xtor(name);
        if &on_typ.name != expected {
            return Err(TypeError::NotInType {
                expected: expected.clone(),
                actual: on_typ.name.clone(),
                span: on_typ.info.span.to_miette(),
            });
        }

        params.infer_telescope(prg, ctx, |ctx, params_out| {
            let on_typ_out = on_typ.infer(prg, ctx)?;
            let in_typ_out = in_typ.infer(prg, ctx)?;

            Ok(tst::Dtor {
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
impl Infer for ust::Def {
    type Target = tst::Def;

    fn infer(&self, prg: &Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let ust::Def { info, name, params, on_typ, in_typ, body } = self;

        params.infer_telescope(prg, ctx, |ctx, params_out| {
            let on_typ_out = on_typ.infer(prg, ctx)?;
            let in_typ_out = in_typ.infer(prg, ctx)?;
            let body_out = body.with_scrutinee(on_typ.clone()).check(prg, ctx, in_typ.clone())?;
            Ok(tst::Def {
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
impl Infer for ust::Codef {
    type Target = tst::Codef;

    fn infer(&self, prg: &Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let ust::Codef { info, name, params, typ, body } = self;

        params.infer_telescope(prg, ctx, |ctx, params_out| {
            let typ_out = typ.infer(prg, ctx)?;
            let body_out = body.with_scrutinee(typ.clone()).infer(prg, ctx)?;
            Ok(tst::Codef {
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
impl<'a> Check for WithScrutinee<'a, ust::Match> {
    type Target = tst::Match;

    fn check(&self, prg: &Prg, ctx: &mut Ctx, t: Rc<ust::Exp>) -> Result<Self::Target, TypeError> {
        let ust::Match { info, cases } = &self.inner;

        // Check exhaustiveness
        let ctors_expected = prg.xtors_for_typ(&self.scrutinee.name);
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
            return Err(TypeError::invalid_match(
                ctors_missing.cloned().collect(),
                ctors_undeclared.cloned().collect(),
                ctors_duplicate,
                info,
            ));
        }

        // Consider all cases
        let cases_out: Vec<_> = cases
            .iter()
            .map(|case| {
                // Build equations for this case
                let ust::Ctor { typ: ust::TypApp { args: def_args, .. }, .. } =
                    &*prg.ctor(&case.name);
                let ust::TypApp { args: on_args, .. } = &self.scrutinee;
                let on_args = on_args.shift((1, 0)); // FIXME: where to shift this

                let eqns: Vec<_> =
                    on_args.iter().cloned().zip(def_args.iter().cloned()).map(Eqn::from).collect();

                // Check the case given the equations
                case.with_eqns(eqns).check(prg, ctx, t.clone())
            })
            .collect::<Result<_, _>>()?;

        Ok(tst::Match { info: info.clone().into(), cases: cases_out })
    }
}

/// Infer a copattern match
impl<'a> Infer for WithScrutinee<'a, ust::Comatch> {
    type Target = tst::Comatch;

    fn infer(&self, prg: &Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let ust::Comatch { info, cases } = &self.inner;

        // Check exhaustiveness
        let dtors_expected = prg.xtors_for_typ(&self.scrutinee.name);
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
            return Err(TypeError::invalid_match(
                dtors_missing.cloned().collect(),
                dtors_exessive.cloned().collect(),
                dtors_duplicate,
                info,
            ));
        }

        // Consider all cases
        let cases_out: Vec<_> = cases
            .iter()
            .map(|case| {
                // Build equations for this case
                let ust::Dtor { on_typ: ust::TypApp { args: def_args, .. }, in_typ, .. } =
                    &*prg.dtor(&case.name);

                let ust::TypApp { args: on_args, .. } = &self.scrutinee;
                let on_args = on_args.shift((1, 0)); // FIXME: where to shift this

                let eqns: Vec<_> =
                    on_args.iter().cloned().zip(def_args.iter().cloned()).map(Eqn::from).collect();

                // Check the case given the equations
                case.with_eqns(eqns).check(prg, ctx, in_typ.clone())
            })
            .collect::<Result<_, _>>()?;

        Ok(tst::Comatch { info: info.clone().into(), cases: cases_out })
    }
}

/// Infer a case in a pattern match
impl<'a> Check for WithEqns<'a, ust::Case> {
    type Target = tst::Case;

    #[trace("{:P} |- {:P} <= {:P}", ctx, self.inner, t)]
    fn check(&self, prg: &Prg, ctx: &mut Ctx, t: Rc<ust::Exp>) -> Result<Self::Target, TypeError> {
        let ust::Case { info, name, args, body } = self.inner;
        let ust::Ctor { name, params, .. } = &*prg.ctor(name);

        args.check_telescope(prg, name, ctx, params, |ctx, args_out| {
            let body_out = match body {
                Some(body) => {
                    let unif = unify(ctx.levels(), self.eqns.clone())
                        .map_err(TypeError::Unify)?
                        .map_no(|()| TypeError::PatternIsAbsurd {
                            name: name.clone(),
                            span: info.span.to_miette(),
                        })
                        .ok_yes()?;

                    // FIXME: Track substitution in context instead
                    let mut ctx = ctx.subst(&mut ctx.levels(), &unif);
                    let body = body.subst(&mut ctx.levels(), &unif);
                    let ctx = &mut ctx;

                    let body_out =
                        body.check(prg, ctx, t.shift((1, 0)).subst(&mut ctx.levels(), &unif))?;

                    Some(body_out)
                }
                None => {
                    unify(ctx.levels(), self.eqns.clone())
                        .map_err(TypeError::Unify)?
                        .map_yes(|_| TypeError::PatternIsNotAbsurd {
                            name: name.clone(),
                            span: info.span.to_miette(),
                        })
                        .ok_no()?;

                    None
                }
            };

            Ok(tst::Case {
                info: info.clone().into(),
                name: name.clone(),
                args: args_out,
                body: body_out,
            })
        })
    }
}

/// Infer a cocase in a co-pattern match
impl<'a> Check for WithEqns<'a, ust::Cocase> {
    type Target = tst::Cocase;

    #[trace("{:P} |- {:P} <= {:P}", ctx, self.inner, t)]
    fn check(&self, prg: &Prg, ctx: &mut Ctx, t: Rc<ust::Exp>) -> Result<Self::Target, TypeError> {
        let ust::Cocase { info, name, args, body } = self.inner;
        let ust::Dtor { name, params, .. } = &*prg.dtor(name);

        args.check_telescope(prg, name, ctx, params, |ctx, args_out| {
            let body_out = match body {
                Some(body) => {
                    let unif = unify(ctx.levels(), self.eqns.clone())
                        .map_err(TypeError::Unify)?
                        .map_no(|()| TypeError::PatternIsAbsurd {
                            name: name.clone(),
                            span: info.span.to_miette(),
                        })
                        .ok_yes()?;

                    // FIXME: Track substitution in context instead
                    let mut ctx = ctx.subst(&mut ctx.levels(), &unif);
                    let body = body.subst(&mut ctx.levels(), &unif);
                    let ctx = &mut ctx;

                    let body_out = body.check(prg, ctx, t.subst(&mut ctx.levels(), &unif))?;

                    Some(body_out)
                }
                None => {
                    unify(ctx.levels(), self.eqns.clone())
                        .map_err(TypeError::Unify)?
                        .map_yes(|_| TypeError::PatternIsNotAbsurd {
                            name: name.clone(),
                            span: info.span.to_miette(),
                        })
                        .ok_no()?;

                    None
                }
            };

            Ok(tst::Cocase {
                info: info.clone().into(),
                name: name.clone(),
                args: args_out,
                body: body_out,
            })
        })
    }
}

/// Check an expression
impl Check for ust::Exp {
    type Target = tst::Exp;

    #[trace("{:P} |- {:P} <= {:P}", ctx, self, t)]
    fn check(&self, prg: &Prg, ctx: &mut Ctx, t: Rc<ust::Exp>) -> Result<Self::Target, TypeError> {
        let out = match self {
            ust::Exp::Match { info, name, on_exp, in_typ: (), body } => {
                let on_exp_out = on_exp.infer(prg, ctx)?;
                let on_typ_out = on_exp_out.typ().check(
                    prg,
                    ctx,
                    Rc::new(ust::Exp::Type { info: ust::Info::empty() }),
                )?;
                let typ_app = on_typ_out.expect_typ_app()?;
                let body_out = body.with_scrutinee(typ_app.forget()).check(prg, ctx, t.clone())?;
                let in_typ_out =
                    t.check(prg, ctx, Rc::new(ust::Exp::Type { info: ust::Info::empty() }))?;

                tst::Exp::Match {
                    info: info.with_type_app(typ_app),
                    name: name.clone(),
                    on_exp: on_exp_out,
                    in_typ: in_typ_out.into(),
                    body: body_out,
                }
            }
            ust::Exp::Comatch { info, name, body } => {
                let t_out =
                    t.check(prg, ctx, Rc::new(ust::Exp::Type { info: ust::Info::empty() }))?;
                let typ_app = t_out.expect_typ_app()?;
                let body_out = body.with_scrutinee(typ_app.forget()).infer(prg, ctx)?;

                tst::Exp::Comatch {
                    info: info.with_type_app(typ_app),
                    name: name.clone(),
                    body: body_out,
                }
            }
            _ => {
                let actual = self.infer(prg, ctx)?;
                actual.typ().convert(&t)?;
                actual
            }
        };

        Ok(out)
    }
}

/// Infer an expression
impl Infer for ust::Exp {
    type Target = tst::Exp;

    #[trace("{:P} |- {:P} => {return:P}", ctx, self, |ret| ret.as_ref().map(|e| e.typ()))]
    fn infer(&self, prg: &Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        match self {
            ust::Exp::Var { info, name, idx } => {
                let typ = ctx.lookup(*idx);
                Ok(tst::Exp::Var { info: info.with_type(typ), name: name.clone(), idx: *idx })
            }
            ust::Exp::TypCtor { info, name, args } => {
                let ust::TypAbs { params } = &*prg.typ(name);

                let args_out = args.check_args(prg, name, ctx, params)?;

                Ok(tst::Exp::TypCtor {
                    info: info.with_type(Rc::new(ust::Exp::Type { info: ust::Info::empty() })),
                    name: name.clone(),
                    args: args_out,
                })
            }
            ust::Exp::Ctor { info, name, args } => {
                let ust::Ctor { name, params, typ, .. } = &*prg.ctor(name);

                let args_out = args.check_args(prg, name, ctx, params)?;
                let typ_out = typ.subst_under_telescope(params, args).to_exp();

                Ok(tst::Exp::Ctor {
                    info: info.with_type(Rc::new(typ_out)),
                    name: name.clone(),
                    args: args_out,
                })
            }
            ust::Exp::Dtor { info, exp, name, args } => {
                let ust::Dtor { name, params, on_typ, in_typ, .. } = &*prg.dtor(name);

                let args_out = args.check_args(prg, name, ctx, params)?;

                let on_typ_out = on_typ.subst_under_telescope(params, args);
                let typ_out = in_typ.subst_under_telescope(params, args);

                let exp_out = exp.check(prg, ctx, Rc::new(on_typ_out.to_exp()))?;

                Ok(tst::Exp::Dtor {
                    info: info.with_type(typ_out),
                    exp: exp_out,
                    name: name.clone(),
                    args: args_out,
                })
            }
            ust::Exp::Anno { info, exp, typ } => {
                let typ_out =
                    typ.check(prg, ctx, Rc::new(ust::Exp::Type { info: ust::Info::empty() }))?;
                let exp_out = (**exp).check(prg, ctx, typ.clone())?;
                Ok(tst::Exp::Anno {
                    info: info.with_type(typ.clone()),
                    exp: Rc::new(exp_out),
                    typ: typ_out,
                })
            }
            ust::Exp::Type { info } => Ok(tst::Exp::Type {
                info: info.with_type(Rc::new(ust::Exp::Type { info: ust::Info::empty() })),
            }),
            _ => Err(TypeError::AnnotationRequired { span: self.info().span.to_miette() }),
        }
    }
}

impl Infer for ust::TypApp {
    type Target = tst::TypApp;

    fn infer(&self, prg: &Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let ust::TypApp { info, name, args } = self;
        let ust::TypAbs { params } = &*prg.typ(name);

        let args_out = args.check_args(prg, name, ctx, params)?;
        Ok(tst::TypApp {
            info: info.with_type(Rc::new(ust::Exp::Type { info: ust::Info::empty() })),
            name: name.clone(),
            args: args_out,
        })
    }
}

impl CheckArgs for ust::Args {
    type Target = tst::Args;

    fn check_args(
        &self,
        prg: &Prg,
        name: &String,
        ctx: &mut Ctx,
        params: &ust::Telescope,
    ) -> Result<Self::Target, TypeError> {
        if self.len() != params.len() {
            return Err(TypeError::ArgLenMismatch {
                name: name.to_string(),
                expected: params.len(),
                actual: self.len(),
            });
        }

        let ust::Telescope { params } = params.subst_in_telescope(self);

        self.iter()
            .zip(params)
            .map(|(exp, ust::Param { typ, .. })| exp.check(prg, ctx, typ))
            .collect()
    }
}

impl CheckTelescope for ust::TelescopeInst {
    type Target = tst::TelescopeInst;

    fn check_telescope<T, F: FnOnce(&mut Ctx, Self::Target) -> Result<T, TypeError>>(
        &self,
        prg: &Prg,
        name: &String,
        ctx: &mut Ctx,
        param_types: &ust::Telescope,
        f: F,
    ) -> Result<T, TypeError> {
        let ust::Telescope { params: param_types } = param_types;
        let ust::TelescopeInst { params } = self;

        if params.len() != param_types.len() {
            return Err(TypeError::ArgLenMismatch {
                name: name.to_string(),
                expected: param_types.len(),
                actual: params.len(),
            });
        }

        let iter = params.iter().zip(param_types);

        ctx.bind_fold(
            iter,
            Result::<_, TypeError>::Ok(vec![]),
            |ctx, params_out, (param_actual, param_expected)| {
                let ust::ParamInst { info, name, typ: () } = param_actual;
                let ust::Param { typ, .. } = param_expected;
                let typ_out =
                    typ.check(prg, ctx, Rc::new(ust::Exp::Type { info: ust::Info::empty() }))?;
                let mut params_out = params_out?;
                let param_out = tst::ParamInst {
                    info: info.with_type(typ.clone()),
                    name: name.clone(),
                    typ: typ_out.into(),
                };
                params_out.push(param_out);
                Ok(params_out)
            },
            |ctx, params| f(ctx, tst::TelescopeInst { params: params? }),
        )
    }
}

impl InferTelescope for ust::Telescope {
    type Target = tst::Telescope;

    fn infer_telescope<T, F: FnOnce(&mut Ctx, Self::Target) -> Result<T, TypeError>>(
        &self,
        prg: &Prg,
        ctx: &mut Ctx,
        f: F,
    ) -> Result<T, TypeError> {
        let ust::Telescope { params } = self;

        ctx.bind_fold(
            params.iter(),
            Result::<_, TypeError>::Ok(tst::Params::new()),
            |ctx, params_out, param| {
                let ust::Param { typ, name } = param;
                let mut params_out = params_out?;
                let typ_out = typ.infer(prg, ctx)?;
                let param_out = tst::Param { name: name.clone(), typ: typ_out };
                params_out.push(param_out);
                Ok(params_out)
            },
            |ctx, params| f(ctx, tst::Telescope { params: params? }),
        )
    }
}

trait SubstUnderTelescope {
    /// Substitute under a telescope
    fn subst_under_telescope(&self, telescope: &ust::Telescope, args: &[Rc<ust::Exp>]) -> Self;
}

impl<T: Substitutable<ust::UST> + Clone> SubstUnderTelescope for T {
    fn subst_under_telescope(&self, telescope: &ust::Telescope, args: &[Rc<ust::Exp>]) -> Self {
        let ust::Telescope { params } = telescope;

        let in_exp = self.clone();

        Ctx::empty().bind_fold(
            params.iter(),
            (),
            |_, _, _| (),
            |ctx, _| in_exp.subst(&mut ctx.levels(), &args),
        )
    }
}

trait SubstInTelescope {
    /// Substitute in a telescope
    fn subst_in_telescope(&self, args: &[Rc<ust::Exp>]) -> Self;
}

impl SubstInTelescope for ust::Telescope {
    fn subst_in_telescope(&self, args: &[Rc<ust::Exp>]) -> Self {
        let ust::Telescope { params } = self;

        Ctx::empty().bind_fold(
            params.iter(),
            Vec::new(),
            |ctx, mut params_out, param| {
                params_out.push(param.subst(&mut ctx.levels(), &args));
                params_out
            },
            |_, params_out| ust::Telescope { params: params_out },
        )
    }
}

trait ExpectTypApp {
    fn expect_typ_app(&self) -> Result<tst::TypApp, TypeError>;
}

impl ExpectTypApp for Rc<tst::Exp> {
    fn expect_typ_app(&self) -> Result<tst::TypApp, TypeError> {
        match &**self {
            tst::Exp::TypCtor { info, name, args } => {
                Ok(tst::TypApp { info: info.clone(), name: name.clone(), args: args.clone() })
            }
            _ => Err(TypeError::expected_typ_app(self.clone())),
        }
    }
}

impl Convert for Rc<ust::Exp> {
    #[trace("{:P} =? {:P}", self, other)]
    fn convert(&self, other: &Self) -> Result<(), TypeError> {
        self.alpha_eq(other)
            .then_some(())
            .ok_or_else(|| TypeError::not_eq(self.clone(), other.clone()))
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

    fn check(&self, prg: &Prg, ctx: &mut Ctx, t: Rc<ust::Exp>) -> Result<Self::Target, TypeError> {
        Ok(Rc::new((**self).check(prg, ctx, t)?))
    }
}

impl<T: Infer> Infer for Rc<T> {
    type Target = Rc<T::Target>;

    fn infer(&self, prg: &Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        Ok(Rc::new((**self).infer(prg, ctx)?))
    }
}
