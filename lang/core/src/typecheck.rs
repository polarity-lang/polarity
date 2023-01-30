//! Bidirectional type checker

use std::rc::Rc;

use data::HashSet;
use miette_util::ToMiette;
use syntax::common::*;
use syntax::ctx::{Bind, BindElem, Context, LevelCtx};
use syntax::nf;
use syntax::tst::{self, ElabInfoExt, HasTypeInfo};
use syntax::ust;
use tracer::trace;

use crate::eval::Eval;
use crate::normalize::Normalize;
use crate::read_back::ReadBack;

use super::ctx::*;
use super::result::TypeError;
use super::unify::*;

pub fn check(prg: &ust::Prg) -> Result<tst::Prg, TypeError> {
    let mut var_ctx = Ctx::empty();
    let mut ng = empty_name_gen();
    prg.infer(prg, &mut var_ctx, &mut ng)
}

pub trait Infer {
    type Target;

    fn infer(
        &self,
        prg: &ust::Prg,
        ctx: &mut Ctx,
        name_gen: &mut NameGen,
    ) -> Result<Self::Target, TypeError>;
}

pub trait Check {
    type Target;

    fn check(
        &self,
        prg: &ust::Prg,
        ctx: &mut Ctx,
        name_gen: &mut NameGen,
        t: Rc<nf::Nf>,
    ) -> Result<Self::Target, TypeError>;
}

pub trait CheckArgs {
    type Target;

    fn check_args(
        &self,
        prg: &ust::Prg,
        name: &str,
        ctx: &mut Ctx,
        ng: &mut NameGen,
        params: &ust::Telescope,
    ) -> Result<Self::Target, TypeError>;
}

pub trait CheckTelescope {
    type Target;

    fn check_telescope<T, F: FnOnce(&mut Ctx, Self::Target) -> Result<T, TypeError>>(
        &self,
        prg: &ust::Prg,
        name: &str,
        ctx: &mut Ctx,
        ng: &mut NameGen,
        params: &ust::Telescope,
        f: F,
    ) -> Result<T, TypeError>;
}

pub trait InferTelescope {
    type Target;

    fn infer_telescope<T, F: FnOnce(&mut Ctx, Self::Target) -> Result<T, TypeError>>(
        &self,
        prg: &ust::Prg,
        ctx: &mut Ctx,
        ng: &mut NameGen,
        f: F,
    ) -> Result<T, TypeError>;
}

pub trait CheckEqns {
    type Target;

    fn check_eqns<T, F: FnOnce(&mut Ctx, Self::Target) -> Result<T, TypeError>>(
        &self,
        ctx: &mut Ctx,
        ng: &mut NameGen,
        eqns: &[Eqn],
        f: F,
    ) -> Result<T, TypeError>;
}

pub trait Convert {
    fn convert(&self, other: &Self) -> Result<(), TypeError>;
}

struct WithScrutinee<'a, T> {
    inner: &'a T,
    scrutinee: nf::TypApp,
}

trait WithScrutineeExt: Sized {
    fn with_scrutinee(&self, scrutinee: nf::TypApp) -> WithScrutinee<'_, Self>;
}

impl<T> WithScrutineeExt for T {
    fn with_scrutinee(&self, scrutinee: nf::TypApp) -> WithScrutinee<'_, Self> {
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

    fn infer(
        &self,
        prg: &ust::Prg,
        ctx: &mut Ctx,
        ng: &mut NameGen,
    ) -> Result<Self::Target, TypeError> {
        let ust::Prg { decls, exp } = self;

        let decls_out = decls.infer(prg, ctx, ng)?;
        let exp_out = exp.as_ref().map(|exp| exp.infer(prg, ctx, ng)).transpose()?;

        Ok(tst::Prg { decls: decls_out, exp: exp_out })
    }
}

/// Infer all declarations in a program
impl Infer for ust::Decls {
    type Target = tst::Decls;

    fn infer(
        &self,
        prg: &ust::Prg,
        ctx: &mut Ctx,
        ng: &mut NameGen,
    ) -> Result<Self::Target, TypeError> {
        let ust::Decls { map, source } = self;

        // FIXME: Reconsider order

        let map_out = map
            .iter()
            .map(|(name, decl)| Ok((name.clone(), decl.infer(prg, ctx, ng)?)))
            .collect::<Result<_, TypeError>>()?;

        Ok(tst::Decls { map: map_out, source: source.clone() })
    }
}

/// Infer a declaration
impl Infer for ust::Decl {
    type Target = tst::Decl;

    #[trace("{:P} |- {} =>", ctx, self.name())]
    fn infer(
        &self,
        prg: &ust::Prg,
        ctx: &mut Ctx,
        ng: &mut NameGen,
    ) -> Result<Self::Target, TypeError> {
        let out = match self {
            ust::Decl::Data(data) => tst::Decl::Data(data.infer(prg, ctx, ng)?),
            ust::Decl::Codata(codata) => tst::Decl::Codata(codata.infer(prg, ctx, ng)?),
            ust::Decl::Ctor(ctor) => tst::Decl::Ctor(ctor.infer(prg, ctx, ng)?),
            ust::Decl::Dtor(dtor) => tst::Decl::Dtor(dtor.infer(prg, ctx, ng)?),
            ust::Decl::Def(def) => tst::Decl::Def(def.infer(prg, ctx, ng)?),
            ust::Decl::Codef(codef) => tst::Decl::Codef(codef.infer(prg, ctx, ng)?),
        };
        Ok(out)
    }
}

/// Infer a data declaration
impl Infer for ust::Data {
    type Target = tst::Data;

    fn infer(
        &self,
        prg: &ust::Prg,
        ctx: &mut Ctx,
        ng: &mut NameGen,
    ) -> Result<Self::Target, TypeError> {
        let ust::Data { info, name, typ, ctors, impl_block } = self;

        let typ_out = typ.infer(prg, ctx, ng)?;

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

    fn infer(
        &self,
        prg: &ust::Prg,
        ctx: &mut Ctx,
        ng: &mut NameGen,
    ) -> Result<Self::Target, TypeError> {
        let ust::Codata { info, name, typ, dtors, impl_block } = self;

        let typ_out = typ.infer(prg, ctx, ng)?;

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

    fn infer(
        &self,
        prg: &ust::Prg,
        ctx: &mut Ctx,
        ng: &mut NameGen,
    ) -> Result<Self::Target, TypeError> {
        let ust::TypAbs { params } = self;

        params.infer_telescope(prg, ctx, ng, |_, params_out| Ok(tst::TypAbs { params: params_out }))
    }
}

/// Infer a constructor declaration
impl Infer for ust::Ctor {
    type Target = tst::Ctor;

    fn infer(
        &self,
        prg: &ust::Prg,
        ctx: &mut Ctx,
        ng: &mut NameGen,
    ) -> Result<Self::Target, TypeError> {
        let ust::Ctor { info, name, params, typ } = self;

        // Check that the constructor lies in the data type it is defined in
        let type_decl = prg.decls.type_decl_for_member(name);
        let expected = type_decl.name();
        if &typ.name != expected {
            return Err(TypeError::NotInType {
                expected: expected.clone(),
                actual: typ.name.clone(),
                span: typ.info.span.to_miette(),
            });
        }

        params.infer_telescope(prg, ctx, ng, |ctx, params_out| {
            let typ_out = typ.infer(prg, ctx, ng)?;

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

    fn infer(
        &self,
        prg: &ust::Prg,
        ctx: &mut Ctx,
        ng: &mut NameGen,
    ) -> Result<Self::Target, TypeError> {
        let ust::Dtor { info, name, params, self_param, ret_typ } = self;

        // Check that the destructor lies in the codata type it is defined in
        let type_decl = prg.decls.type_decl_for_member(name);
        let expected = type_decl.name();
        if &self_param.typ.name != expected {
            return Err(TypeError::NotInType {
                expected: expected.clone(),
                actual: self_param.typ.name.clone(),
                span: self_param.typ.info.span.to_miette(),
            });
        }

        params.infer_telescope(prg, ctx, ng, |ctx, params_out| {
            self_param.infer_telescope(prg, ctx, ng, |ctx, self_param_out| {
                let ret_typ_out = ret_typ.infer(prg, ctx, ng)?;

                Ok(tst::Dtor {
                    info: info.clone().into(),
                    name: name.clone(),
                    params: params_out,
                    self_param: self_param_out,
                    ret_typ: ret_typ_out,
                })
            })
        })
    }
}

/// Infer a definition
impl Infer for ust::Def {
    type Target = tst::Def;

    fn infer(
        &self,
        prg: &ust::Prg,
        ctx: &mut Ctx,
        ng: &mut NameGen,
    ) -> Result<Self::Target, TypeError> {
        let ust::Def { info, name, params, self_param, ret_typ, body } = self;

        params.infer_telescope(prg, ctx, ng, |ctx, params_out| {
            let self_param_nf = self_param.typ.normalize(prg, &mut ctx.env())?;

            let (ret_typ_out, ret_typ_nf, self_param_out) =
                self_param.infer_telescope(prg, ctx, ng, |ctx, self_param_out| {
                    let ret_typ_out = ret_typ.infer(prg, ctx, ng)?;
                    let ret_typ_nf = ret_typ.normalize(prg, &mut ctx.env())?;
                    Ok((ret_typ_out, ret_typ_nf, self_param_out))
                })?;

            let body_out = body.with_scrutinee(self_param_nf).check(prg, ctx, ng, ret_typ_nf)?;
            Ok(tst::Def {
                info: info.clone().into(),
                name: name.clone(),
                params: params_out,
                self_param: self_param_out,
                ret_typ: ret_typ_out,
                body: body_out,
            })
        })
    }
}

/// Infer a co-definition
impl Infer for ust::Codef {
    type Target = tst::Codef;

    fn infer(
        &self,
        prg: &ust::Prg,
        ctx: &mut Ctx,
        ng: &mut NameGen,
    ) -> Result<Self::Target, TypeError> {
        let ust::Codef { info, name, params, typ, body } = self;

        params.infer_telescope(prg, ctx, ng, |ctx, params_out| {
            let typ_out = typ.infer(prg, ctx, ng)?;
            let typ_nf = typ.normalize(prg, &mut ctx.env())?;
            let body_out = body.with_scrutinee(typ_nf).infer(prg, ctx, ng)?;
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

    fn check(
        &self,
        prg: &ust::Prg,
        ctx: &mut Ctx,
        ng: &mut NameGen,
        t: Rc<nf::Nf>,
    ) -> Result<Self::Target, TypeError> {
        let ust::Match { info, cases } = &self.inner;

        // Check that this match is on a data type
        let ust::Type::Data(data) = prg.decls.typ(&self.scrutinee.name) else {
            return Err(TypeError::ComatchOnData {
                name: self.scrutinee.name.clone(),
                span: info.span.to_miette()
            });
        };

        // Check exhaustiveness
        let ctors_expected: HashSet<_> = data.ctors.iter().cloned().collect();
        let mut ctors_actual = HashSet::default();
        let mut ctors_duplicate = HashSet::default();

        for name in cases.iter().map(|case| &case.name) {
            if ctors_actual.contains(name) {
                ctors_duplicate.insert(name.clone());
            }
            ctors_actual.insert(name.clone());
        }
        let mut ctors_missing = ctors_expected.difference(&ctors_actual).peekable();
        let mut ctors_undeclared = ctors_actual.difference(&ctors_expected).peekable();

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
                let ust::Ctor { typ: ust::TypApp { args: def_args, .. }, params, .. } =
                    prg.decls.ctor(&case.name);

                let def_args_nf = LevelCtx::empty().bind_iter(params.params.iter(), |ctx| {
                    def_args.normalize(prg, &mut ctx.env())
                })?;

                let nf::TypApp { args: on_args, .. } = &self.scrutinee;
                let on_args = on_args.shift((1, 0)); // FIXME: where to shift this

                let eqns: Vec<_> = def_args_nf
                    .iter()
                    .cloned()
                    .zip(on_args.iter().cloned())
                    .map(Eqn::from)
                    .collect();

                // TODO: Substitute the constructor for the self parameter
                // TODO: Adjust local matches to work again
                let t = t.shift((-1, 0));

                // Check the case given the equations
                case.with_eqns(eqns).check(prg, ctx, ng, t)
            })
            .collect::<Result<_, _>>()?;

        Ok(tst::Match { info: info.clone().into(), cases: cases_out })
    }
}

/// Infer a copattern match
impl<'a> Infer for WithScrutinee<'a, ust::Comatch> {
    type Target = tst::Comatch;

    fn infer(
        &self,
        prg: &ust::Prg,
        ctx: &mut Ctx,
        ng: &mut NameGen,
    ) -> Result<Self::Target, TypeError> {
        let ust::Comatch { info, cases } = &self.inner;

        // Check that this comatch is on a codata type
        let ust::Type::Codata(codata) = prg.decls.typ(&self.scrutinee.name) else {
            return Err(TypeError::ComatchOnData {
                name: self.scrutinee.name.clone(),
                span: info.span.to_miette()
            });
        };

        // Check exhaustiveness
        let dtors_expected: HashSet<_> = codata.dtors.iter().cloned().collect();
        let mut dtors_actual = HashSet::default();
        let mut dtors_duplicate = HashSet::default();

        for name in cases.iter().map(|case| &case.name) {
            if dtors_actual.contains(name) {
                dtors_duplicate.insert(name.clone());
            }
            dtors_actual.insert(name.clone());
        }

        let mut dtors_missing = dtors_expected.difference(&dtors_actual).peekable();
        let mut dtors_exessive = dtors_actual.difference(&dtors_expected).peekable();

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
                let ust::Dtor {
                    self_param: ust::SelfParam { typ: ust::TypApp { args: def_args, .. }, .. },
                    ret_typ,
                    params,
                    ..
                } = prg.decls.dtor(&case.name);

                let def_args_nf = LevelCtx::empty().bind_iter(params.params.iter(), |ctx| {
                    def_args.normalize(prg, &mut ctx.env())
                })?;

                let ret_typ_nf =
                    ret_typ.normalize(prg, &mut LevelCtx::from(vec![params.len(), 1]).env())?;

                let nf::TypApp { args: on_args, .. } = &self.scrutinee;
                let on_args = on_args.shift((1, 0)); // FIXME: where to shift this

                let eqns: Vec<_> = def_args_nf
                    .iter()
                    .cloned()
                    .zip(on_args.iter().cloned())
                    .map(Eqn::from)
                    .collect();

                // TODO: Substitute the comatch label for the self parameter
                let ret_typ_nf = ret_typ_nf.shift((-1, 0));

                // Check the case given the equations
                case.with_eqns(eqns)
                    .with_scrutinee(self.scrutinee.shift((1, 0)))
                    .check(prg, ctx, ng, ret_typ_nf)
            })
            .collect::<Result<_, _>>()?;

        Ok(tst::Comatch { info: info.clone().into(), cases: cases_out })
    }
}

/// Infer a case in a pattern match
impl<'a> Check for WithEqns<'a, ust::Case> {
    type Target = tst::Case;

    #[trace("{:P} |- {:P} <= {:P}", ctx, self.inner, t)]
    fn check(
        &self,
        prg: &ust::Prg,
        ctx: &mut Ctx,
        ng: &mut NameGen,
        t: Rc<nf::Nf>,
    ) -> Result<Self::Target, TypeError> {
        let ust::Case { info, name, args, body } = self.inner;
        let ust::Ctor { name, params, .. } = prg.decls.ctor(name);

        args.check_telescope(prg, name, ctx, ng, params, |ctx, args_out| {
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
                    let mut ctx = ctx.subst(prg, &unif)?;
                    let body = body.subst(&mut ctx.levels(), &unif);
                    let ctx = &mut ctx;

                    let t_subst = t.forget().shift((1, 0)).subst(&mut ctx.levels(), &unif);
                    let t_nf = t_subst.normalize(prg, &mut ctx.env())?;

                    let body_out = body.check(prg, ctx, ng, t_nf)?;

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
impl<'a> Check for WithScrutinee<'a, WithEqns<'a, ust::Cocase>> {
    type Target = tst::Cocase;

    #[trace("{:P} |- {:P} <= {:P}", ctx, self.inner.inner, t)]
    fn check(
        &self,
        prg: &ust::Prg,
        ctx: &mut Ctx,
        ng: &mut NameGen,
        t: Rc<nf::Nf>,
    ) -> Result<Self::Target, TypeError> {
        let ust::Cocase { info, name, params: params_inst, body } = self.inner.inner;
        let ust::Dtor { name, params, .. } = prg.decls.dtor(name);

        params_inst.check_telescope(prg, name, ctx, ng, params, |ctx, args_out| {
            let body_out = match body {
                Some(body) => {
                    let unif = unify(ctx.levels(), self.inner.eqns.clone())
                        .map_err(TypeError::Unify)?
                        .map_no(|()| TypeError::PatternIsAbsurd {
                            name: name.clone(),
                            span: info.span.to_miette(),
                        })
                        .ok_yes()?;

                    // FIXME: Track substitution in context instead
                    let mut ctx = ctx.subst(prg, &unif)?;
                    let body = body.subst(&mut ctx.levels(), &unif);
                    let ctx = &mut ctx;

                    let t_subst = t.forget().subst(&mut ctx.levels(), &unif);
                    let t_nf = t_subst.normalize(prg, &mut ctx.env())?;

                    let body_out = body.check(prg, ctx, ng, t_nf)?;

                    Some(body_out)
                }
                None => {
                    unify(ctx.levels(), self.inner.eqns.clone())
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
                params: args_out,
                body: body_out,
            })
        })
    }
}

/// Check an expression
impl Check for ust::Exp {
    type Target = tst::Exp;

    #[trace("{:P} |- {:P} <= {:P}", ctx, self, t)]
    fn check(
        &self,
        prg: &ust::Prg,
        ctx: &mut Ctx,
        ng: &mut NameGen,
        t: Rc<nf::Nf>,
    ) -> Result<Self::Target, TypeError> {
        let out = match self {
            ust::Exp::Match { info, name, on_exp, ret_typ: (), body } => {
                let on_exp_out = on_exp.infer(prg, ctx, ng)?;
                let typ_app_nf = on_exp_out.typ().expect_typ_app()?;
                let typ_app = typ_app_nf.forget().infer(prg, ctx, ng)?;
                let body_out =
                    body.with_scrutinee(typ_app_nf.clone()).check(prg, ctx, ng, t.clone())?;
                let ret_typ_out = t.forget().check(prg, ctx, ng, type_univ())?;
                let typ_name = typ_app.name;
                let name = name.unwrap_or({
                    let name_id = ng.names.get(&typ_name).cloned().unwrap_or(0);
                    ng.names.insert(typ_name, name_id + 1);
                    format!("{typ_name}{}", (name_id.to_string()))
                });

                tst::Exp::Match {
                    info: info.with_type_app(typ_app, typ_app_nf),
                    name: name.clone(),
                    on_exp: on_exp_out,
                    ret_typ: ret_typ_out.into(),
                    body: body_out,
                }
            }
            ust::Exp::Comatch { info, name, body } => {
                let typ_app_nf = t.expect_typ_app()?;
                let typ_app = typ_app_nf.forget().infer(prg, ctx, ng)?;
                let body_out = body.with_scrutinee(typ_app_nf.clone()).infer(prg, ctx, ng)?;
                let ty_name = typ_app.name;
                let typ_name = typ_app.name;
                let name = name.unwrap_or({
                    let name_id = ng.names.get(&typ_name).cloned().unwrap_or(0);
                    ng.names.insert(typ_name, name_id + 1);
                    format!("{typ_name}{}", (name_id.to_string()))
                });

                tst::Exp::Comatch {
                    info: info.with_type_app(typ_app, typ_app_nf),
                    name: name.clone(),
                    body: body_out,
                }
            }
            _ => {
                let actual = self.infer(prg, ctx, ng)?;
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
    fn infer(
        &self,
        prg: &ust::Prg,
        ctx: &mut Ctx,
        ng: &mut NameGen,
    ) -> Result<Self::Target, TypeError> {
        match self {
            ust::Exp::Var { info, name, idx } => {
                let typ = ctx.lookup(*idx);
                let typ_nf = typ.read_back(prg)?;
                Ok(tst::Exp::Var { info: info.with_type(typ_nf), name: name.clone(), idx: *idx })
            }
            ust::Exp::TypCtor { info, name, args } => {
                let ust::TypAbs { params } = &*prg.decls.typ(name).typ();

                let args_out = args.check_args(prg, name, ctx, ng, params)?;

                Ok(tst::Exp::TypCtor {
                    info: info.with_type(type_univ()),
                    name: name.clone(),
                    args: args_out,
                })
            }
            ust::Exp::Ctor { info, name, args } => {
                let ust::Ctor { name, params, typ, .. } = &prg.decls.ctor_or_codef(name);

                let args_out = args.check_args(prg, name, ctx, ng, params)?;
                let typ_out = typ.subst_under_ctx(vec![params.len()].into(), &&[args][..]).to_exp();
                let typ_nf = typ_out.normalize(prg, &mut ctx.env())?;

                Ok(tst::Exp::Ctor {
                    info: info.with_type(typ_nf),
                    name: name.clone(),
                    args: args_out,
                })
            }
            ust::Exp::Dtor { info, exp, name, args } => {
                let ust::Dtor { name, params, self_param, ret_typ, .. } =
                    &prg.decls.dtor_or_def(name);

                let args_out = args.check_args(prg, name, ctx, ng, params)?;

                let self_param_out = self_param
                    .typ
                    .subst_under_ctx(vec![params.len()].into(), &&[args][..])
                    .to_exp();
                let self_param_nf = self_param_out.normalize(prg, &mut ctx.env())?;
                let subst = [&args[..], &[Rc::new(self_param.typ.to_exp())][..]];
                let typ_out = ret_typ.subst_under_ctx(vec![params.len(), 1].into(), &&subst[..]);
                let typ_out_nf = typ_out.normalize(prg, &mut ctx.env())?;

                let exp_out = exp.check(prg, ctx, ng, self_param_nf)?;

                Ok(tst::Exp::Dtor {
                    info: info.with_type(typ_out_nf),
                    exp: exp_out,
                    name: name.clone(),
                    args: args_out,
                })
            }
            ust::Exp::Anno { info, exp, typ } => {
                let typ_out = typ.check(prg, ctx, ng, type_univ())?;
                let typ_nf = typ.normalize(prg, &mut ctx.env())?;
                let exp_out = (**exp).check(prg, ctx, ng, typ_nf.clone())?;
                Ok(tst::Exp::Anno {
                    info: info.with_type(typ_nf),
                    exp: Rc::new(exp_out),
                    typ: typ_out,
                })
            }
            ust::Exp::Type { info } => Ok(tst::Exp::Type { info: info.with_type(type_univ()) }),
            _ => Err(TypeError::AnnotationRequired { span: self.info().span.to_miette() }),
        }
    }
}

impl Infer for ust::TypApp {
    type Target = tst::TypApp;

    fn infer(
        &self,
        prg: &ust::Prg,
        ctx: &mut Ctx,
        ng: &mut NameGen,
    ) -> Result<Self::Target, TypeError> {
        let ust::TypApp { info, name, args } = self;
        let ust::TypAbs { params } = &*prg.decls.typ(name).typ();

        let args_out = args.check_args(prg, name, ctx, ng, params)?;
        Ok(tst::TypApp { info: info.with_type(type_univ()), name: name.clone(), args: args_out })
    }
}

impl CheckArgs for ust::Args {
    type Target = tst::Args;

    fn check_args(
        &self,
        prg: &ust::Prg,
        name: &str,
        ctx: &mut Ctx,
        ng: &mut NameGen,
        params: &ust::Telescope,
    ) -> Result<Self::Target, TypeError> {
        if self.len() != params.len() {
            return Err(TypeError::ArgLenMismatch {
                name: name.to_owned(),
                expected: params.len(),
                actual: self.len(),
            });
        }

        let ust::Telescope { params } = params.subst_in_telescope(LevelCtx::empty(), &&[self][..]);

        self.iter()
            .zip(params)
            .map(|(exp, ust::Param { typ, .. })| {
                let typ = typ.normalize(prg, &mut ctx.env())?;
                exp.check(prg, ctx, ng, typ)
            })
            .collect()
    }
}

impl CheckTelescope for ust::TelescopeInst {
    type Target = tst::TelescopeInst;

    fn check_telescope<T, F: FnOnce(&mut Ctx, Self::Target) -> Result<T, TypeError>>(
        &self,
        prg: &ust::Prg,
        name: &str,
        ctx: &mut Ctx,
        ng: &mut NameGen,
        param_types: &ust::Telescope,
        f: F,
    ) -> Result<T, TypeError> {
        let ust::Telescope { params: param_types } = param_types;
        let ust::TelescopeInst { params } = self;

        if params.len() != param_types.len() {
            return Err(TypeError::ArgLenMismatch {
                name: name.to_owned(),
                expected: param_types.len(),
                actual: params.len(),
            });
        }

        let iter = params.iter().zip(param_types);

        ctx.bind_fold_failable(
            iter,
            vec![],
            |ctx, params_out, (param_actual, param_expected)| {
                let ust::ParamInst { info, name, typ: () } = param_actual;
                let ust::Param { typ, .. } = param_expected;
                let typ_out = typ.check(prg, ctx, ng, type_univ())?;
                let typ_val = typ.eval(prg, &mut ctx.env())?;
                let typ_nf = typ_val.read_back(prg)?;
                let mut params_out = params_out;
                let param_out = tst::ParamInst {
                    info: info.with_type(typ_nf),
                    name: name.clone(),
                    typ: typ_out.into(),
                };
                params_out.push(param_out);
                Result::<_, TypeError>::Ok(BindElem { elem: typ_val, ret: params_out })
            },
            |ctx, params| f(ctx, tst::TelescopeInst { params }),
        )?
    }
}

impl InferTelescope for ust::Telescope {
    type Target = tst::Telescope;

    fn infer_telescope<T, F: FnOnce(&mut Ctx, Self::Target) -> Result<T, TypeError>>(
        &self,
        prg: &ust::Prg,
        ctx: &mut Ctx,
        ng: &mut NameGen,
        f: F,
    ) -> Result<T, TypeError> {
        let ust::Telescope { params } = self;

        ctx.bind_fold_failable(
            params.iter(),
            tst::Params::new(),
            |ctx, mut params_out, param| {
                let ust::Param { typ, name } = param;
                let typ_out = typ.check(prg, ctx, ng, type_univ())?;
                let elem = typ.eval(prg, &mut ctx.env())?;
                let param_out = tst::Param { name: name.clone(), typ: typ_out };
                params_out.push(param_out);
                Result::<_, TypeError>::Ok(BindElem { elem, ret: params_out })
            },
            |ctx, params| f(ctx, tst::Telescope { params }),
        )?
    }
}

impl InferTelescope for ust::SelfParam {
    type Target = tst::SelfParam;

    fn infer_telescope<T, F: FnOnce(&mut Ctx, Self::Target) -> Result<T, TypeError>>(
        &self,
        prg: &ust::Prg,
        ctx: &mut Ctx,
        ng: &mut NameGen,
        f: F,
    ) -> Result<T, TypeError> {
        let ust::SelfParam { info, name, typ } = self;

        let elem = typ.to_exp().eval(prg, &mut ctx.env())?;
        let typ_out = typ.infer(prg, ctx, ng)?;
        let param_out = tst::SelfParam {
            info: tst::Info { span: info.span },
            name: name.clone(),
            typ: typ_out,
        };

        ctx.bind_single(&elem, |ctx| f(ctx, param_out))
    }
}

// trait SubstUnderTelescope {
//     /// Substitute under a telescope
//     fn subst_under_telescope(&self, telescope: &ust::Telescope, args: &[Rc<ust::Exp>]) -> Self;
// }

// impl<T: Substitutable<Rc<ust::Exp>> + Clone> SubstUnderTelescope for T {
//     fn subst_under_telescope(&self, telescope: &ust::Telescope, args: &[Rc<ust::Exp>]) -> Self {
//         let ust::Telescope { params } = telescope;

//         LevelCtx::empty().bind_fold(
//             params.iter(),
//             (),
//             |_, _, _| (),
//             |ctx, _| self.subst(ctx, &args),
//         )
//     }
// }

trait SubstUnderCtx {
    fn subst_under_ctx<S: Substitution<Rc<ust::Exp>>>(&self, ctx: LevelCtx, s: &S) -> Self;
}

impl<T: Substitutable<Rc<ust::Exp>> + Clone> SubstUnderCtx for T {
    fn subst_under_ctx<S: Substitution<Rc<ust::Exp>>>(&self, mut ctx: LevelCtx, s: &S) -> Self {
        self.subst(&mut ctx, s)
    }
}

trait SubstInTelescope {
    /// Substitute in a telescope
    fn subst_in_telescope<S: Substitution<Rc<ust::Exp>>>(&self, ctx: LevelCtx, s: &S) -> Self;
}

impl SubstInTelescope for ust::Telescope {
    fn subst_in_telescope<S: Substitution<Rc<ust::Exp>>>(&self, mut ctx: LevelCtx, s: &S) -> Self {
        let ust::Telescope { params } = self;

        ctx.bind_fold(
            params.iter(),
            Vec::new(),
            |ctx, mut params_out, param| {
                params_out.push(param.subst(ctx, s));
                params_out
            },
            |_, params_out| ust::Telescope { params: params_out },
        )
    }
}

trait ExpectTypApp {
    fn expect_typ_app(&self) -> Result<nf::TypApp, TypeError>;
}

impl ExpectTypApp for Rc<nf::Nf> {
    fn expect_typ_app(&self) -> Result<nf::TypApp, TypeError> {
        match &**self {
            nf::Nf::TypCtor { info, name, args } => {
                Ok(nf::TypApp { info: info.clone(), name: name.clone(), args: args.clone() })
            }
            _ => Err(TypeError::expected_typ_app(self.clone())),
        }
    }
}

impl Convert for Rc<nf::Nf> {
    #[trace("{:P} =? {:P}", self, other)]
    fn convert(&self, other: &Self) -> Result<(), TypeError> {
        self.alpha_eq(other)
            .then_some(())
            .ok_or_else(|| TypeError::not_eq(self.clone(), other.clone()))
    }
}

impl<T: Check> Check for Rc<T> {
    type Target = Rc<T::Target>;

    fn check(
        &self,
        prg: &ust::Prg,
        ctx: &mut Ctx,
        ng: &mut NameGen,
        t: Rc<nf::Nf>,
    ) -> Result<Self::Target, TypeError> {
        Ok(Rc::new((**self).check(prg, ctx, ng, t)?))
    }
}

impl<T: Infer> Infer for Rc<T> {
    type Target = Rc<T::Target>;

    fn infer(
        &self,
        prg: &ust::Prg,
        ctx: &mut Ctx,
        ng: &mut NameGen,
    ) -> Result<Self::Target, TypeError> {
        Ok(Rc::new((**self).infer(prg, ctx, ng)?))
    }
}

fn type_univ() -> Rc<nf::Nf> {
    Rc::new(nf::Nf::Type { info: ust::Info::empty() })
}
