//! Bidirectional type checker

use std::rc::Rc;

use codespan::Span;

use crate::normalizer::env::ToEnv;
use crate::normalizer::normalize::Normalize;
use crate::unifier::unify::*;
use miette_util::ToMiette;
use syntax::common::*;
use syntax::ctx::values::Binder;
use syntax::ctx::{BindContext, BindElem, LevelCtx};
use syntax::generic::{Hole, Named, TypeUniv, Variable};
use syntax::tst::forget::ForgetTST;
use syntax::tst::{self, HasTypeInfo};
use syntax::ust::util::Instantiate;
use syntax::ust::{self, Occurs};
use tracer::trace;

use super::ctx::*;
use crate::result::TypeError;

pub fn check(prg: &ust::Prg) -> Result<tst::Prg, TypeError> {
    let mut var_ctx = Ctx::default();
    prg.infer(prg, &mut var_ctx)
}

pub trait Infer {
    type Target;

    fn infer(&self, prg: &ust::Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError>;
}

pub trait Check {
    type Target;

    fn check(
        &self,
        prg: &ust::Prg,
        ctx: &mut Ctx,
        t: Rc<ust::Exp>,
    ) -> Result<Self::Target, TypeError>;
}

pub trait CheckTelescope {
    type Target;

    fn check_telescope<T, F: FnOnce(&mut Ctx, Self::Target) -> Result<T, TypeError>>(
        &self,
        prg: &ust::Prg,
        name: &str,
        ctx: &mut Ctx,
        params: &ust::Telescope,
        f: F,
        span: Option<Span>,
    ) -> Result<T, TypeError>;
}

pub trait InferTelescope {
    type Target;

    fn infer_telescope<T, F: FnOnce(&mut Ctx, Self::Target) -> Result<T, TypeError>>(
        &self,
        prg: &ust::Prg,
        ctx: &mut Ctx,
        f: F,
    ) -> Result<T, TypeError>;
}

impl Infer for ust::Prg {
    type Target = tst::Prg;

    fn infer(&self, prg: &ust::Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let ust::Prg { decls } = self;

        let decls_out = decls.infer(prg, ctx)?;

        Ok(tst::Prg { decls: decls_out })
    }
}

/// Infer all declarations in a program
impl Infer for ust::Decls {
    type Target = tst::Decls;

    fn infer(&self, prg: &ust::Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let ust::Decls { map, lookup_table } = self;

        // FIXME: Reconsider order

        let map_out = map
            .iter()
            .map(|(name, decl)| Ok((name.clone(), decl.infer(prg, ctx)?)))
            .collect::<Result<_, TypeError>>()?;

        Ok(tst::Decls { map: map_out, lookup_table: lookup_table.clone() })
    }
}

/// Infer a declaration
impl Infer for ust::Decl {
    type Target = tst::Decl;

    #[trace("{:P} |- {} =>", ctx, self.name())]
    fn infer(&self, prg: &ust::Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let out = match self {
            ust::Decl::Data(data) => tst::Decl::Data(data.infer(prg, ctx)?),
            ust::Decl::Codata(codata) => tst::Decl::Codata(codata.infer(prg, ctx)?),
            ust::Decl::Ctor(ctor) => tst::Decl::Ctor(ctor.infer(prg, ctx)?),
            ust::Decl::Dtor(dtor) => tst::Decl::Dtor(dtor.infer(prg, ctx)?),
            ust::Decl::Def(def) => tst::Decl::Def(def.infer(prg, ctx)?),
            ust::Decl::Codef(codef) => tst::Decl::Codef(codef.infer(prg, ctx)?),
            ust::Decl::Let(tl_let) => tst::Decl::Let(tl_let.infer(prg, ctx)?),
        };
        Ok(out)
    }
}

/// Infer a data declaration
impl Infer for ust::Data {
    type Target = tst::Data;

    fn infer(&self, prg: &ust::Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let ust::Data { span, doc, name, attr, typ, ctors } = self;

        let typ_out = typ.infer(prg, ctx)?;

        Ok(tst::Data {
            span: *span,
            doc: doc.clone(),
            name: name.clone(),
            attr: attr.clone(),
            typ: typ_out,
            ctors: ctors.clone(),
        })
    }
}

/// Infer a codata declaration
impl Infer for ust::Codata {
    type Target = tst::Codata;

    fn infer(&self, prg: &ust::Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let ust::Codata { span, doc, name, attr, typ, dtors } = self;

        let typ_out = typ.infer(prg, ctx)?;

        Ok(tst::Codata {
            span: *span,
            doc: doc.clone(),
            name: name.clone(),
            attr: attr.clone(),
            typ: typ_out,
            dtors: dtors.clone(),
        })
    }
}

/// Infer a codata declaration
impl Infer for ust::TypAbs {
    type Target = tst::TypAbs;

    fn infer(&self, prg: &ust::Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let ust::TypAbs { params } = self;

        params.infer_telescope(prg, ctx, |_, params_out| Ok(tst::TypAbs { params: params_out }))
    }
}

/// Infer a constructor declaration
impl Infer for ust::Ctor {
    type Target = tst::Ctor;

    fn infer(&self, prg: &ust::Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let ust::Ctor { span, doc, name, params, typ } = self;

        // Check that the constructor lies in the data type it is defined in
        let data_type = prg.decls.data_for_ctor(name, *span)?;
        let expected = &data_type.name;
        if &typ.name != expected {
            return Err(TypeError::NotInType {
                expected: expected.clone(),
                actual: typ.name.clone(),
                span: typ.span.to_miette(),
            });
        }

        params.infer_telescope(prg, ctx, |ctx, params_out| {
            let typ_out = typ.infer(prg, ctx)?;

            Ok(tst::Ctor {
                span: *span,
                doc: doc.clone(),
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

    fn infer(&self, prg: &ust::Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let ust::Dtor { span, doc, name, params, self_param, ret_typ } = self;

        // Check that the destructor lies in the codata type it is defined in
        let codata_type = prg.decls.codata_for_dtor(name, *span)?;
        let expected = &codata_type.name;
        if &self_param.typ.name != expected {
            return Err(TypeError::NotInType {
                expected: expected.clone(),
                actual: self_param.typ.name.clone(),
                span: self_param.typ.span.to_miette(),
            });
        }

        params.infer_telescope(prg, ctx, |ctx, params_out| {
            self_param.infer_telescope(prg, ctx, |ctx, self_param_out| {
                let ret_typ_out = ret_typ.infer(prg, ctx)?;

                Ok(tst::Dtor {
                    span: *span,
                    doc: doc.clone(),
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

    fn infer(&self, prg: &ust::Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let ust::Def { span, doc, name, attr, params, self_param, ret_typ, body } = self;

        params.infer_telescope(prg, ctx, |ctx, params_out| {
            let self_param_nf = self_param.typ.normalize(prg, &mut ctx.env())?;

            let (ret_typ_out, ret_typ_nf, self_param_out) =
                self_param.infer_telescope(prg, ctx, |ctx, self_param_out| {
                    let ret_typ_out = ret_typ.infer(prg, ctx)?;
                    let ret_typ_nf = ret_typ.normalize(prg, &mut ctx.env())?;
                    Ok((ret_typ_out, ret_typ_nf, self_param_out))
                })?;

            let body_out =
                WithScrutinee { inner: body, scrutinee: self_param_nf.expect_typ_app()? }
                    .check(prg, ctx, ret_typ_nf)?;
            Ok(tst::Def {
                span: *span,
                doc: doc.clone(),
                name: name.clone(),
                attr: attr.clone(),
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

    fn infer(&self, prg: &ust::Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let ust::Codef { span, doc, name, attr, params, typ, body } = self;

        params.infer_telescope(prg, ctx, |ctx, params_out| {
            let typ_out = typ.infer(prg, ctx)?;
            let typ_nf = typ.normalize(prg, &mut ctx.env())?;
            let wd = WithDestructee {
                inner: body,
                label: Some(name.to_owned()),
                n_label_args: params.len(),
                destructee: typ_nf.expect_typ_app()?,
            };
            let body_out = wd.infer(prg, ctx)?;
            Ok(tst::Codef {
                span: *span,
                doc: doc.clone(),
                name: name.clone(),
                attr: attr.clone(),
                params: params_out,
                typ: typ_out,
                body: body_out,
            })
        })
    }
}

impl Infer for ust::Let {
    type Target = tst::Let;

    fn infer(&self, prg: &ust::Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let ust::Let { span, doc, name, attr, params, typ, body } = self;

        params.infer_telescope(prg, ctx, |ctx, params_out| {
            let typ_out = typ.infer(prg, ctx)?;
            let typ_nf = typ.normalize(prg, &mut ctx.env())?;
            let body_out = body.check(prg, ctx, typ_nf)?;

            Ok(tst::Let {
                span: *span,
                doc: doc.clone(),
                name: name.clone(),
                attr: attr.clone(),
                params: params_out,
                typ: typ_out,
                body: body_out,
            })
        })
    }
}
struct WithScrutinee<'a> {
    inner: &'a ust::Match,
    scrutinee: ust::TypCtor,
}

/// Check a pattern match
impl<'a> Check for WithScrutinee<'a> {
    type Target = tst::Match;

    fn check(
        &self,
        prg: &ust::Prg,
        ctx: &mut Ctx,
        t: Rc<ust::Exp>,
    ) -> Result<Self::Target, TypeError> {
        let ust::Match { span, cases, omit_absurd } = &self.inner;

        // Check that this match is on a data type
        let data = prg.decls.data(&self.scrutinee.name, *span)?;

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

        if (!omit_absurd && ctors_missing.peek().is_some())
            || ctors_undeclared.peek().is_some()
            || !ctors_duplicate.is_empty()
        {
            return Err(TypeError::invalid_match(
                ctors_missing.cloned().collect(),
                ctors_undeclared.cloned().collect(),
                ctors_duplicate,
                span,
            ));
        }

        // Add absurd cases for all omitted constructors
        let mut cases: Vec<_> = cases.iter().cloned().map(|case| (case, false)).collect();

        if *omit_absurd {
            for name in ctors_missing.cloned() {
                let ust::Ctor { params, .. } = prg.decls.ctor(&name, *span)?;

                let case =
                    ust::Case { span: *span, name, params: params.instantiate(), body: None };
                cases.push((case, true));
            }
        }

        let mut cases_out = Vec::new();

        for (case, omit) in cases {
            // Build equations for this case
            let ust::Ctor { typ: ust::TypCtor { args: def_args, .. }, params, .. } =
                prg.decls.ctor(&case.name, case.span)?;

            let def_args_nf = LevelCtx::empty()
                .bind_iter(params.params.iter(), |ctx| def_args.normalize(prg, &mut ctx.env()))?;

            let ust::TypCtor { args: on_args, .. } = &self.scrutinee;
            let on_args = on_args.shift((1, 0)); // FIXME: where to shift this

            let eqns: Vec<_> = def_args_nf
                .iter()
                .cloned()
                .zip(on_args.args.iter().cloned())
                .map(Eqn::from)
                .collect();

            // Check the case given the equations
            let case_out = check_case(eqns, &case, prg, ctx, t.clone())?;

            if !omit {
                cases_out.push(case_out);
            }
        }

        Ok(tst::Match { span: *span, cases: cases_out, omit_absurd: *omit_absurd })
    }
}

struct WithDestructee<'a> {
    inner: &'a ust::Match,
    /// Name of the global codefinition that gets substituted for the destructor's self parameters
    label: Option<ust::Ident>,
    n_label_args: usize,
    destructee: ust::TypCtor,
}

/// Infer a copattern match
impl<'a> Infer for WithDestructee<'a> {
    type Target = tst::Match;

    fn infer(&self, prg: &ust::Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let ust::Match { span, cases, omit_absurd } = &self.inner;

        // Check that this comatch is on a codata type
        let codata = prg.decls.codata(&self.destructee.name, *span)?;

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

        if (!omit_absurd && dtors_missing.peek().is_some())
            || dtors_exessive.peek().is_some()
            || !dtors_duplicate.is_empty()
        {
            return Err(TypeError::invalid_match(
                dtors_missing.cloned().collect(),
                dtors_exessive.cloned().collect(),
                dtors_duplicate,
                span,
            ));
        }

        // Add absurd cases for all omitted destructors
        let mut cases: Vec<_> = cases.iter().cloned().map(|case| (case, false)).collect();

        if *omit_absurd {
            for name in dtors_missing.cloned() {
                let ust::Dtor { params, .. } = prg.decls.dtor(&name, *span)?;

                let case =
                    ust::Case { span: *span, name, params: params.instantiate(), body: None };
                cases.push((case, true));
            }
        }

        let mut cases_out = Vec::new();

        for (case, omit) in cases {
            // Build equations for this case
            let ust::Dtor {
                self_param: ust::SelfParam { typ: ust::TypCtor { args: def_args, .. }, .. },
                ret_typ,
                params,
                ..
            } = prg.decls.dtor(&case.name, case.span)?;

            let def_args_nf =
                def_args.normalize(prg, &mut LevelCtx::from(vec![params.len()]).env())?;

            let ret_typ_nf =
                ret_typ.normalize(prg, &mut LevelCtx::from(vec![params.len(), 1]).env())?;

            let ust::TypCtor { args: on_args, .. } = &self.destructee;
            let on_args = on_args.shift((1, 0)); // FIXME: where to shift this

            let eqns: Vec<_> = def_args_nf
                .iter()
                .cloned()
                .zip(on_args.args.iter().cloned())
                .map(Eqn::from)
                .collect();

            let ret_typ_nf = match &self.label {
                // Substitute the codef label for the self parameter
                Some(label) => {
                    let args = (0..self.n_label_args)
                        .rev()
                        .map(|snd| {
                            ust::Exp::Variable(Variable {
                                span: None,
                                idx: Idx { fst: 2, snd },
                                name: "".to_owned(),
                                inferred_type: None,
                            })
                        })
                        .map(Rc::new)
                        .collect();
                    let ctor = Rc::new(ust::Exp::Call(ust::Call {
                        span: None,
                        info: (),
                        name: label.clone(),
                        args: ust::Args { args },
                    }));
                    let subst = Assign(Lvl { fst: 1, snd: 0 }, ctor);
                    let mut subst_ctx = LevelCtx::from(vec![params.len(), 1]);
                    ret_typ_nf.subst(&mut subst_ctx, &subst).shift((-1, 0)).normalize(
                        prg,
                        &mut LevelCtx::from(vec![self.n_label_args, params.len()]).env(),
                    )?
                }
                // TODO: Self parameter for local comatches
                None => ret_typ_nf.shift((-1, 0)),
            };

            // Check the case given the equations
            let case_out = check_cocase(eqns, &case, prg, ctx, ret_typ_nf)?;

            if !omit {
                cases_out.push(case_out);
            }
        }

        Ok(tst::Match { span: *span, cases: cases_out, omit_absurd: *omit_absurd })
    }
}

/// Infer a case in a pattern match
#[trace("{:P} |- {:P} <= {:P}", ctx, case, t)]
fn check_case(
    eqns: Vec<Eqn>,
    case: &ust::Case,
    prg: &ust::Prg,
    ctx: &mut Ctx,
    t: Rc<ust::Exp>,
) -> Result<tst::Case, TypeError> {
    let ust::Case { span, name, params: args, body } = case;
    let ust::Ctor { name, params, .. } = prg.decls.ctor(name, *span)?;

    // FIXME: Refactor this
    let mut subst_ctx_1 = ctx.levels().append(&vec![1, params.len()].into());
    let mut subst_ctx_2 = ctx.levels().append(&vec![params.len(), 1].into());
    let curr_lvl = subst_ctx_2.len() - 1;

    args.check_telescope(
        prg,
        name,
        ctx,
        params,
        |ctx, args_out| {
            // Substitute the constructor for the self parameter
            let args = (0..params.len())
                .rev()
                .map(|snd| {
                    ust::Exp::Variable(Variable {
                        span: None,
                        idx: Idx { fst: 1, snd },
                        name: "".to_owned(),
                        inferred_type: None,
                    })
                })
                .map(Rc::new)
                .collect();
            let ctor = Rc::new(ust::Exp::Call(ust::Call {
                span: None,
                info: (),
                name: name.clone(),
                args: ust::Args { args },
            }));
            let subst = Assign(Lvl { fst: curr_lvl, snd: 0 }, ctor);

            // FIXME: Refactor this
            let t = t
                .shift((1, 0))
                .swap_with_ctx(&mut subst_ctx_1, curr_lvl, curr_lvl - 1)
                .subst(&mut subst_ctx_2, &subst)
                .shift((-1, 0));

            let body_out = match body {
                Some(body) => {
                    let unif = unify(ctx.levels(), eqns.clone(), false)?
                        .map_no(|()| TypeError::PatternIsAbsurd {
                            name: name.clone(),
                            span: span.to_miette(),
                        })
                        .ok_yes()?;

                    ctx.fork::<Result<_, TypeError>, _>(|ctx| {
                        ctx.subst(prg, &unif)?;
                        let body = body.subst(&mut ctx.levels(), &unif);

                        let t_subst = t.subst(&mut ctx.levels(), &unif);
                        let t_nf = t_subst.normalize(prg, &mut ctx.env())?;

                        let body_out = body.check(prg, ctx, t_nf)?;

                        Ok(Some(body_out))
                    })?
                }
                None => {
                    unify(ctx.levels(), eqns.clone(), false)?
                        .map_yes(|_| TypeError::PatternIsNotAbsurd {
                            name: name.clone(),
                            span: span.to_miette(),
                        })
                        .ok_no()?;

                    None
                }
            };

            Ok(tst::Case { span: *span, name: name.clone(), params: args_out, body: body_out })
        },
        *span,
    )
}

/// Infer a cocase in a co-pattern match
#[trace("{:P} |- {:P} <= {:P}", ctx, cocase, t)]
fn check_cocase(
    eqns: Vec<Eqn>,
    cocase: &ust::Case,
    prg: &ust::Prg,
    ctx: &mut Ctx,
    t: Rc<ust::Exp>,
) -> Result<tst::Case, TypeError> {
    let ust::Case { span, name, params: params_inst, body } = cocase;
    let ust::Dtor { name, params, .. } = prg.decls.dtor(name, *span)?;

    params_inst.check_telescope(
        prg,
        name,
        ctx,
        params,
        |ctx, args_out| {
            let body_out = match body {
                Some(body) => {
                    let unif = unify(ctx.levels(), eqns.clone(), false)?
                        .map_no(|()| TypeError::PatternIsAbsurd {
                            name: name.clone(),
                            span: span.to_miette(),
                        })
                        .ok_yes()?;

                    ctx.fork::<Result<_, TypeError>, _>(|ctx| {
                        ctx.subst(prg, &unif)?;
                        let body = body.subst(&mut ctx.levels(), &unif);

                        let t_subst = t.subst(&mut ctx.levels(), &unif);
                        let t_nf = t_subst.normalize(prg, &mut ctx.env())?;

                        let body_out = body.check(prg, ctx, t_nf)?;

                        Ok(Some(body_out))
                    })?
                }
                None => {
                    unify(ctx.levels(), eqns.clone(), false)?
                        .map_yes(|_| TypeError::PatternIsNotAbsurd {
                            name: name.clone(),
                            span: span.to_miette(),
                        })
                        .ok_no()?;

                    None
                }
            };

            Ok(tst::Case { span: *span, name: name.clone(), params: args_out, body: body_out })
        },
        *span,
    )
}

/// Check an expression
impl Check for ust::Exp {
    type Target = tst::Exp;

    #[trace("{:P} |- {:P} <= {:P}", ctx, self, t)]
    fn check(
        &self,
        prg: &ust::Prg,
        ctx: &mut Ctx,
        t: Rc<ust::Exp>,
    ) -> Result<Self::Target, TypeError> {
        match self {
            ust::Exp::LocalMatch(e) => Ok(tst::Exp::LocalMatch(e.check(prg, ctx, t.clone())?)),
            ust::Exp::LocalComatch(e) => {
                Ok(tst::Exp::LocalComatch(e.check(prg, ctx, t.clone())?))
            }
            ust::Exp::Hole(e) => Ok(tst::Exp::Hole(e.check(prg, ctx, t.clone())?)),
            _ => {
                let inferred_term = self.infer(prg, ctx)?;
                let inferred_typ = inferred_term.typ().ok_or(TypeError::Impossible {
                    message: "Expected inferred type".to_owned(),
                    span: None,
                })?;
                let ctx = ctx.levels();
                convert(ctx, inferred_typ, &t)?;
                Ok(inferred_term)
            }
        }
    }
}

impl Check for ust::LocalMatch {
    type Target = tst::LocalMatch;

    fn check(
        &self,
        prg: &ust::Prg,
        ctx: &mut Ctx,
        t: Rc<ust::Exp>,
    ) -> Result<Self::Target, TypeError> {
        let ust::LocalMatch { span, info: (), ctx: _, name, on_exp, motive, ret_typ: _, body } =
            self;
        let on_exp_out = on_exp.infer(prg, ctx)?;
        let typ_app_nf = on_exp_out
            .typ()
            .ok_or(TypeError::Impossible {
                message: "Expected inferred type".to_owned(),
                span: None,
            })?
            .expect_typ_app()?;
        let typ_app = typ_app_nf.infer(prg, ctx)?;
        let ret_typ_out = t.check(prg, ctx, type_univ())?;

        let motive_out;
        let body_t;

        match motive {
            // Pattern matching with motive
            Some(m) => {
                let ust::Motive { span: info, param, ret_typ } = m;
                let self_t_nf =
                    typ_app.to_exp().forget_tst().normalize(prg, &mut ctx.env())?.shift((1, 0));
                let self_binder = Binder { name: param.name.clone(), typ: self_t_nf.clone() };

                // Typecheck the motive
                let ret_typ_out =
                    ctx.bind_single(&self_binder, |ctx| ret_typ.check(prg, ctx, type_univ()))?;

                // Ensure that the motive matches the expected type
                let mut subst_ctx = ctx.levels().append(&vec![1].into());
                let on_exp_shifted = on_exp.shift((1, 0));
                let subst = Assign(Lvl { fst: subst_ctx.len() - 1, snd: 0 }, on_exp_shifted);
                let motive_t = ret_typ.subst(&mut subst_ctx, &subst).shift((-1, 0));
                let motive_t_nf = motive_t.normalize(prg, &mut ctx.env())?;
                convert(subst_ctx, motive_t_nf, &t)?;

                body_t =
                    ctx.bind_single(&self_binder, |ctx| ret_typ.normalize(prg, &mut ctx.env()))?;
                motive_out = Some(tst::Motive {
                    span: *info,
                    param: tst::ParamInst {
                        span: *info,
                        info: tst::TypeInfo { typ: self_t_nf, ctx: None },
                        name: param.name.clone(),
                        typ: Rc::new(typ_app.to_exp()).into(),
                    },
                    ret_typ: ret_typ_out,
                });
            }
            // Pattern matching without motive
            None => {
                body_t = t.shift((1, 0));
                motive_out = None;
            }
        };

        let body_out =
            WithScrutinee { inner: body, scrutinee: typ_app_nf.clone() }.check(prg, ctx, body_t)?;

        Ok(tst::LocalMatch {
            span: *span,
            info: tst::TypeAppInfo { typ: typ_app, typ_nf: typ_app_nf },
            ctx: Some(ctx.vars.clone()),
            name: name.clone(),
            on_exp: on_exp_out,
            motive: motive_out,
            ret_typ: ret_typ_out.into(),
            body: body_out,
        })
    }
}

impl Check for ust::LocalComatch {
    type Target = tst::LocalComatch;

    fn check(
        &self,
        prg: &ust::Prg,
        ctx: &mut Ctx,
        t: Rc<ust::Exp>,
    ) -> Result<Self::Target, TypeError> {
        let ust::LocalComatch { span, info: (), ctx: _, name, is_lambda_sugar, body } = self;
        let typ_app_nf = t.expect_typ_app()?;
        let typ_app = typ_app_nf.infer(prg, ctx)?;

        // Local comatches don't support self parameters, yet.
        let codata = prg.decls.codata(&typ_app.name, *span)?;
        if uses_self(prg, codata)? {
            return Err(TypeError::LocalComatchWithSelf {
                type_name: codata.name.to_owned(),
                span: span.to_miette(),
            });
        }

        let wd = WithDestructee {
            inner: body,
            label: None,
            n_label_args: 0,
            destructee: typ_app_nf.clone(),
        };
        let body_out = wd.infer(prg, ctx)?;

        Ok(tst::LocalComatch {
            span: *span,
            info: tst::TypeAppInfo { typ: typ_app, typ_nf: typ_app_nf },
            ctx: Some(ctx.vars.clone()),
            name: name.clone(),
            is_lambda_sugar: *is_lambda_sugar,
            body: body_out,
        })
    }
}

impl Check for Hole {
    type Target = Hole;

    fn check(
        &self,
        _prg: &ust::Prg,
        ctx: &mut Ctx,
        t: Rc<ust::Exp>,
    ) -> Result<Self::Target, TypeError> {
        let Hole { span, .. } = self;
        Ok(Hole {
            span: *span,
            inferred_type: Some(t.clone()),
            inferred_ctx: Some(ctx.vars.clone()),
        })
    }
}

/// Infer an expression
impl Infer for ust::Exp {
    type Target = tst::Exp;

    #[trace("{:P} |- {:P} => {return:P}", ctx, self, |ret| ret.as_ref().map(|e| e.typ()))]
    fn infer(&self, prg: &ust::Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        match self {
            ust::Exp::Variable(e) => Ok(tst::Exp::Variable(e.infer(prg, ctx)?)),
            ust::Exp::TypCtor(e) => Ok(tst::Exp::TypCtor(e.infer(prg, ctx)?)),
            ust::Exp::Call(e) => Ok(tst::Exp::Call(e.infer(prg, ctx)?)),
            ust::Exp::DotCall(e) => Ok(tst::Exp::DotCall(e.infer(prg, ctx)?)),
            ust::Exp::Anno(e) => Ok(tst::Exp::Anno(e.infer(prg, ctx)?)),
            ust::Exp::TypeUniv(e) => Ok(tst::Exp::TypeUniv(e.infer(prg, ctx)?)),
            ust::Exp::Hole(e) => Ok(tst::Exp::Hole(e.infer(prg, ctx)?)),
            ust::Exp::LocalMatch(e) => Ok(tst::Exp::LocalMatch(e.infer(prg, ctx)?)),
            ust::Exp::LocalComatch(e) => Ok(tst::Exp::LocalComatch(e.infer(prg, ctx)?)),
        }
    }
}

impl Infer for Variable {
    type Target = Variable;

    fn infer(&self, _prg: &ust::Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let Variable { span, idx, name, .. } = self;
        let typ_nf = ctx.lookup(*idx);
        Ok(Variable { span: *span, idx: *idx, name: name.clone(), inferred_type: Some(typ_nf) })
    }
}

impl Infer for ust::TypCtor {
    type Target = tst::TypCtor;

    fn infer(&self, prg: &ust::Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let ust::TypCtor { span, info: (), name, args } = self;
        let ust::TypAbs { params } = &*prg.decls.typ(name, *span)?.typ();
        let args_out = check_args(args, prg, name, ctx, params, *span)?;

        Ok(tst::TypCtor {
            span: *span,
            info: tst::TypeInfo { typ: type_univ(), ctx: None },
            name: name.clone(),
            args: args_out,
        })
    }
}

impl Infer for ust::Call {
    type Target = tst::Call;

    fn infer(&self, prg: &ust::Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let ust::Call { span, info: (), name, args } = self;
        let ust::Ctor { name, params, typ, .. } = &prg.decls.ctor_or_codef(name, *span)?;
        let args_out = check_args(args, prg, name, ctx, params, *span)?;
        let typ_out =
            typ.subst_under_ctx(vec![params.len()].into(), &vec![args.args.clone()]).to_exp();
        let typ_nf = typ_out.normalize(prg, &mut ctx.env())?;
        Ok(tst::Call {
            span: *span,
            info: tst::TypeInfo { typ: typ_nf, ctx: None },
            name: name.clone(),
            args: args_out,
        })
    }
}

impl Infer for ust::DotCall {
    type Target = tst::DotCall;

    fn infer(&self, prg: &ust::Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let ust::DotCall { span, info: (), exp, name, args } = self;
        let ust::Dtor { name, params, self_param, ret_typ, .. } =
            &prg.decls.dtor_or_def(name, *span)?;

        let args_out = check_args(args, prg, name, ctx, params, *span)?;

        let self_param_out = self_param
            .typ
            .subst_under_ctx(vec![params.len()].into(), &vec![args.args.clone()])
            .to_exp();
        let self_param_nf = self_param_out.normalize(prg, &mut ctx.env())?;

        let exp_out = exp.check(prg, ctx, self_param_nf)?;

        let subst = vec![args.args.clone(), vec![exp.clone()]];
        let typ_out = ret_typ.subst_under_ctx(vec![params.len(), 1].into(), &subst);
        let typ_out_nf = typ_out.normalize(prg, &mut ctx.env())?;

        Ok(tst::DotCall {
            span: *span,
            info: tst::TypeInfo { typ: typ_out_nf, ctx: None },
            exp: exp_out,
            name: name.clone(),
            args: args_out,
        })
    }
}

impl Infer for ust::Anno {
    type Target = tst::Anno;

    fn infer(&self, prg: &ust::Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let ust::Anno { span, info: (), exp, typ } = self;
        let typ_out = typ.check(prg, ctx, type_univ())?;
        let typ_nf = typ.normalize(prg, &mut ctx.env())?;
        let exp_out = (**exp).check(prg, ctx, typ_nf.clone())?;
        Ok(tst::Anno {
            span: *span,
            info: tst::TypeInfo { typ: typ_nf, ctx: None },
            exp: Rc::new(exp_out),
            typ: typ_out,
        })
    }
}

impl Infer for TypeUniv {
    type Target = TypeUniv;

    fn infer(&self, _prg: &ust::Prg, _ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        Ok(self.clone())
    }
}

impl Infer for Hole {
    type Target = Hole;

    fn infer(&self, _prg: &ust::Prg, _ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let Hole { span, .. } = self;
        Ok(Hole { span: *span, inferred_type: Some(type_hole()), inferred_ctx: None })
    }
}

impl Infer for ust::LocalMatch {
    type Target = tst::LocalMatch;

    fn infer(&self, _prg: &ust::Prg, _ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        Err(TypeError::CannotInferMatch { span: self.span().to_miette() })
    }
}

impl Infer for ust::LocalComatch {
    type Target = tst::LocalComatch;

    fn infer(&self, _prg: &ust::Prg, _ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        Err(TypeError::CannotInferComatch { span: self.span().to_miette() })
    }
}

fn check_args(
    this: &ust::Args,
    prg: &ust::Prg,
    name: &str,
    ctx: &mut Ctx,
    params: &ust::Telescope,
    span: Option<Span>,
) -> Result<tst::Args, TypeError> {
    if this.len() != params.len() {
        return Err(TypeError::ArgLenMismatch {
            name: name.to_owned(),
            expected: params.len(),
            actual: this.len(),
            span: span.to_miette(),
        });
    }

    let ust::Telescope { params } =
        params.subst_in_telescope(LevelCtx::empty(), &vec![this.args.clone()]);

    let args = this
        .args
        .iter()
        .zip(params)
        .map(|(exp, ust::Param { typ, .. })| {
            let typ = typ.normalize(prg, &mut ctx.env())?;
            exp.check(prg, ctx, typ)
        })
        .collect::<Result<_, _>>()?;

    Ok(tst::Args { args })
}

impl CheckTelescope for ust::TelescopeInst {
    type Target = tst::TelescopeInst;

    fn check_telescope<T, F: FnOnce(&mut Ctx, Self::Target) -> Result<T, TypeError>>(
        &self,
        prg: &ust::Prg,
        name: &str,
        ctx: &mut Ctx,
        param_types: &ust::Telescope,
        f: F,
        span: Option<Span>,
    ) -> Result<T, TypeError> {
        let ust::Telescope { params: param_types } = param_types;
        let ust::TelescopeInst { params } = self;

        if params.len() != param_types.len() {
            return Err(TypeError::ArgLenMismatch {
                name: name.to_owned(),
                expected: param_types.len(),
                actual: params.len(),
                span: span.to_miette(),
            });
        }

        let iter = params.iter().zip(param_types);

        ctx.bind_fold_failable(
            iter,
            vec![],
            |ctx, params_out, (param_actual, param_expected)| {
                let ust::ParamInst { span, info: (), name, typ: _ } = param_actual;
                let ust::Param { typ, .. } = param_expected;
                let typ_out = typ.check(prg, ctx, type_univ())?;
                let typ_nf = typ.normalize(prg, &mut ctx.env())?;
                let mut params_out = params_out;
                let param_out = tst::ParamInst {
                    span: *span,
                    info: tst::TypeInfo { typ: typ_nf.clone(), ctx: None },
                    name: name.clone(),
                    typ: typ_out.into(),
                };
                params_out.push(param_out);
                let elem = Binder { name: param_actual.name.clone(), typ: typ_nf };
                Result::<_, TypeError>::Ok(BindElem { elem, ret: params_out })
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
        f: F,
    ) -> Result<T, TypeError> {
        let ust::Telescope { params } = self;

        ctx.bind_fold_failable(
            params.iter(),
            vec![],
            |ctx, mut params_out, param| {
                let ust::Param { typ, name } = param;
                let typ_out = typ.check(prg, ctx, type_univ())?;
                let typ_nf = typ.normalize(prg, &mut ctx.env())?;
                let param_out = tst::Param { name: name.clone(), typ: typ_out };
                params_out.push(param_out);
                let elem = Binder { name: param.name.clone(), typ: typ_nf };
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
        f: F,
    ) -> Result<T, TypeError> {
        let ust::SelfParam { info, name, typ } = self;

        let typ_nf = typ.to_exp().normalize(prg, &mut ctx.env())?;
        let typ_out = typ.infer(prg, ctx)?;
        let param_out = tst::SelfParam { info: *info, name: name.clone(), typ: typ_out };
        let elem = Binder { name: name.clone().unwrap_or_default(), typ: typ_nf };

        // We need to shift the self parameter type here because we treat it as a 1-element telescope
        ctx.bind_single(&elem.shift((1, 0)), |ctx| f(ctx, param_out))
    }
}

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
    fn expect_typ_app(&self) -> Result<ust::TypCtor, TypeError>;
}

impl ExpectTypApp for Rc<ust::Exp> {
    fn expect_typ_app(&self) -> Result<ust::TypCtor, TypeError> {
        match &**self {
            ust::Exp::TypCtor(ust::TypCtor { span, info, name, args }) => Ok(ust::TypCtor {
                span: *span,
                info: *info,
                name: name.clone(),
                args: args.clone(),
            }),
            _ => Err(TypeError::expected_typ_app(self.clone())),
        }
    }
}

#[trace("{:P} =? {:P}", this, other)]
fn convert(ctx: LevelCtx, this: Rc<ust::Exp>, other: &Rc<ust::Exp>) -> Result<(), TypeError> {
    // Convertibility is checked using the unification algorithm.
    let eqn: Eqn = Eqn { lhs: this.clone(), rhs: other.clone() };
    let eqns: Vec<Eqn> = vec![eqn];
    let res = unify(ctx, eqns, true)?;
    match res {
        crate::unifier::dec::Dec::Yes(_) => Ok(()),
        crate::unifier::dec::Dec::No(_) => Err(TypeError::not_eq(this.clone(), other.clone())),
    }
}

impl<T: Check> Check for Rc<T> {
    type Target = Rc<T::Target>;

    fn check(
        &self,
        prg: &ust::Prg,
        ctx: &mut Ctx,
        t: Rc<ust::Exp>,
    ) -> Result<Self::Target, TypeError> {
        Ok(Rc::new((**self).check(prg, ctx, t)?))
    }
}

impl<T: Infer> Infer for Rc<T> {
    type Target = Rc<T::Target>;

    fn infer(&self, prg: &ust::Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        Ok(Rc::new((**self).infer(prg, ctx)?))
    }
}

fn type_univ() -> Rc<ust::Exp> {
    Rc::new(ust::Exp::TypeUniv(TypeUniv { span: None }))
}

fn type_hole() -> Rc<ust::Exp> {
    Rc::new(ust::Exp::Hole(Hole { span: None, inferred_type: None, inferred_ctx: None }))
}

// Checks whether the codata type contains destructors with a self parameter
fn uses_self(prg: &ust::Prg, codata: &ust::Codata) -> Result<bool, TypeError> {
    for dtor_name in &codata.dtors {
        let dtor = prg.decls.dtor(dtor_name, None)?;
        let mut ctx = LevelCtx::from(vec![dtor.params.len(), 1]);
        if dtor.ret_typ.occurs(&mut ctx, Lvl { fst: 1, snd: 0 }) {
            return Ok(true);
        }
    }
    Ok(false)
}
