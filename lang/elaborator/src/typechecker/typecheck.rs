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
use syntax::generic::*;
use tracer::trace;

use super::ctx::*;
use super::subst::{SubstInTelescope, SubstUnderCtx};
use super::util::*;
use crate::result::TypeError;

pub fn check(prg: &Prg) -> Result<Prg, TypeError> {
    let mut var_ctx = Ctx::default();
    prg.infer(prg, &mut var_ctx)
}

pub trait Infer {
    type Target;

    fn infer(&self, prg: &Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError>;
}

pub trait Check {
    type Target;

    fn check(&self, prg: &Prg, ctx: &mut Ctx, t: Rc<Exp>) -> Result<Self::Target, TypeError>;
}

pub trait CheckTelescope {
    type Target;

    fn check_telescope<T, F: FnOnce(&mut Ctx, Self::Target) -> Result<T, TypeError>>(
        &self,
        prg: &Prg,
        name: &str,
        ctx: &mut Ctx,
        params: &Telescope,
        f: F,
        span: Option<Span>,
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

impl Infer for Prg {
    type Target = Prg;

    fn infer(&self, prg: &Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let Prg { decls } = self;

        let decls_out = decls.infer(prg, ctx)?;

        Ok(Prg { decls: decls_out })
    }
}

/// Infer all declarations in a program
impl Infer for Decls {
    type Target = Decls;

    fn infer(&self, prg: &Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let Decls { map, lookup_table } = self;

        // FIXME: Reconsider order

        let map_out = map
            .iter()
            .map(|(name, decl)| Ok((name.clone(), decl.infer(prg, ctx)?)))
            .collect::<Result<_, TypeError>>()?;

        Ok(Decls { map: map_out, lookup_table: lookup_table.clone() })
    }
}

/// Infer a declaration
impl Infer for Decl {
    type Target = Decl;

    #[trace("{:P} |- {} =>", ctx, self.name())]
    fn infer(&self, prg: &Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let out = match self {
            Decl::Data(data) => Decl::Data(data.infer(prg, ctx)?),
            Decl::Codata(codata) => Decl::Codata(codata.infer(prg, ctx)?),
            Decl::Ctor(ctor) => Decl::Ctor(ctor.infer(prg, ctx)?),
            Decl::Dtor(dtor) => Decl::Dtor(dtor.infer(prg, ctx)?),
            Decl::Def(def) => Decl::Def(def.infer(prg, ctx)?),
            Decl::Codef(codef) => Decl::Codef(codef.infer(prg, ctx)?),
            Decl::Let(tl_let) => Decl::Let(tl_let.infer(prg, ctx)?),
        };
        Ok(out)
    }
}

/// Infer a data declaration
impl Infer for Data {
    type Target = Data;

    fn infer(&self, prg: &Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let Data { span, doc, name, attr, typ, ctors } = self;

        let typ_out = typ.infer(prg, ctx)?;

        Ok(Data {
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
impl Infer for Codata {
    type Target = Codata;

    fn infer(&self, prg: &Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let Codata { span, doc, name, attr, typ, dtors } = self;

        let typ_out = typ.infer(prg, ctx)?;

        Ok(Codata {
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
impl Infer for TypAbs {
    type Target = TypAbs;

    fn infer(&self, prg: &Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let TypAbs { params } = self;

        params.infer_telescope(prg, ctx, |_, params_out| Ok(TypAbs { params: params_out }))
    }
}

/// Infer a constructor declaration
impl Infer for Ctor {
    type Target = Ctor;

    fn infer(&self, prg: &Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let Ctor { span, doc, name, params, typ } = self;

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

            Ok(Ctor {
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
impl Infer for Dtor {
    type Target = Dtor;

    fn infer(&self, prg: &Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let Dtor { span, doc, name, params, self_param, ret_typ } = self;

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

                Ok(Dtor {
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
impl Infer for Def {
    type Target = Def;

    fn infer(&self, prg: &Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let Def { span, doc, name, attr, params, self_param, ret_typ, body } = self;

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
            Ok(Def {
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
impl Infer for Codef {
    type Target = Codef;

    fn infer(&self, prg: &Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let Codef { span, doc, name, attr, params, typ, body } = self;

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
            Ok(Codef {
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

impl Infer for Let {
    type Target = Let;

    fn infer(&self, prg: &Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let Let { span, doc, name, attr, params, typ, body } = self;

        params.infer_telescope(prg, ctx, |ctx, params_out| {
            let typ_out = typ.infer(prg, ctx)?;
            let typ_nf = typ.normalize(prg, &mut ctx.env())?;
            let body_out = body.check(prg, ctx, typ_nf)?;

            Ok(Let {
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
    inner: &'a Match,
    scrutinee: TypCtor,
}

/// Check a pattern match
impl<'a> Check for WithScrutinee<'a> {
    type Target = Match;

    fn check(&self, prg: &Prg, ctx: &mut Ctx, t: Rc<Exp>) -> Result<Self::Target, TypeError> {
        let Match { span, cases, omit_absurd } = &self.inner;

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
                let Ctor { params, .. } = prg.decls.ctor(&name, *span)?;

                let case = Case { span: *span, name, params: params.instantiate(), body: None };
                cases.push((case, true));
            }
        }

        let mut cases_out = Vec::new();

        for (case, omit) in cases {
            // Build equations for this case
            let Ctor { typ: TypCtor { args: def_args, .. }, params, .. } =
                prg.decls.ctor(&case.name, case.span)?;

            let def_args_nf = LevelCtx::empty()
                .bind_iter(params.params.iter(), |ctx| def_args.normalize(prg, &mut ctx.env()))?;

            let TypCtor { args: on_args, .. } = &self.scrutinee;
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

        Ok(Match { span: *span, cases: cases_out, omit_absurd: *omit_absurd })
    }
}

struct WithDestructee<'a> {
    inner: &'a Match,
    /// Name of the global codefinition that gets substituted for the destructor's self parameters
    label: Option<Ident>,
    n_label_args: usize,
    destructee: TypCtor,
}

/// Infer a copattern match
impl<'a> Infer for WithDestructee<'a> {
    type Target = Match;

    fn infer(&self, prg: &Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let Match { span, cases, omit_absurd } = &self.inner;

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
                let Dtor { params, .. } = prg.decls.dtor(&name, *span)?;

                let case = Case { span: *span, name, params: params.instantiate(), body: None };
                cases.push((case, true));
            }
        }

        let mut cases_out = Vec::new();

        for (case, omit) in cases {
            // Build equations for this case
            let Dtor {
                self_param: SelfParam { typ: TypCtor { args: def_args, .. }, .. },
                ret_typ,
                params,
                ..
            } = prg.decls.dtor(&case.name, case.span)?;

            let def_args_nf =
                def_args.normalize(prg, &mut LevelCtx::from(vec![params.len()]).env())?;

            let ret_typ_nf =
                ret_typ.normalize(prg, &mut LevelCtx::from(vec![params.len(), 1]).env())?;

            let TypCtor { args: on_args, .. } = &self.destructee;
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
                            Exp::Variable(Variable {
                                span: None,
                                idx: Idx { fst: 2, snd },
                                name: "".to_owned(),
                                inferred_type: None,
                            })
                        })
                        .map(Rc::new)
                        .collect();
                    let ctor = Rc::new(Exp::Call(Call {
                        span: None,
                        name: label.clone(),
                        args: Args { args },
                        inferred_type: None,
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

        Ok(Match { span: *span, cases: cases_out, omit_absurd: *omit_absurd })
    }
}

/// Infer a case in a pattern match
#[trace("{:P} |- {:P} <= {:P}", ctx, case, t)]
fn check_case(
    eqns: Vec<Eqn>,
    case: &Case,
    prg: &Prg,
    ctx: &mut Ctx,
    t: Rc<Exp>,
) -> Result<Case, TypeError> {
    let Case { span, name, params: args, body } = case;
    let Ctor { name, params, .. } = prg.decls.ctor(name, *span)?;

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
                    Exp::Variable(Variable {
                        span: None,
                        idx: Idx { fst: 1, snd },
                        name: "".to_owned(),
                        inferred_type: None,
                    })
                })
                .map(Rc::new)
                .collect();
            let ctor = Rc::new(Exp::Call(Call {
                span: None,
                name: name.clone(),
                args: Args { args },
                inferred_type: None,
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

            Ok(Case { span: *span, name: name.clone(), params: args_out, body: body_out })
        },
        *span,
    )
}

/// Infer a cocase in a co-pattern match
#[trace("{:P} |- {:P} <= {:P}", ctx, cocase, t)]
fn check_cocase(
    eqns: Vec<Eqn>,
    cocase: &Case,
    prg: &Prg,
    ctx: &mut Ctx,
    t: Rc<Exp>,
) -> Result<Case, TypeError> {
    let Case { span, name, params: params_inst, body } = cocase;
    let Dtor { name, params, .. } = prg.decls.dtor(name, *span)?;

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

            Ok(Case { span: *span, name: name.clone(), params: args_out, body: body_out })
        },
        *span,
    )
}

/// Check an expression
impl Check for Exp {
    type Target = Exp;

    #[trace("{:P} |- {:P} <= {:P}", ctx, self, t)]
    fn check(&self, prg: &Prg, ctx: &mut Ctx, t: Rc<Exp>) -> Result<Self::Target, TypeError> {
        match self {
            Exp::LocalMatch(e) => Ok(Exp::LocalMatch(e.check(prg, ctx, t.clone())?)),
            Exp::LocalComatch(e) => Ok(Exp::LocalComatch(e.check(prg, ctx, t.clone())?)),
            Exp::Hole(e) => Ok(Exp::Hole(e.check(prg, ctx, t.clone())?)),
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

impl Check for LocalMatch {
    type Target = LocalMatch;

    fn check(&self, prg: &Prg, ctx: &mut Ctx, t: Rc<Exp>) -> Result<Self::Target, TypeError> {
        let LocalMatch { span, name, on_exp, motive, body, .. } = self;
        let on_exp_out = on_exp.infer(prg, ctx)?;
        let typ_app_nf = on_exp_out
            .typ()
            .ok_or(TypeError::Impossible {
                message: "Expected inferred type".to_owned(),
                span: None,
            })?
            .expect_typ_app()?;
        let typ_app = typ_app_nf.infer(prg, ctx)?;
        let ret_typ_out = t.check(prg, ctx, Rc::new(TypeUniv::new().into()))?;

        let motive_out;
        let body_t;

        match motive {
            // Pattern matching with motive
            Some(m) => {
                let Motive { span: info, param, ret_typ } = m;
                let self_t_nf =
                    typ_app.to_exp().forget_tst().normalize(prg, &mut ctx.env())?.shift((1, 0));
                let self_binder = Binder { name: param.name.clone(), typ: self_t_nf.clone() };

                // Typecheck the motive
                let ret_typ_out = ctx.bind_single(&self_binder, |ctx| {
                    ret_typ.check(prg, ctx, Rc::new(TypeUniv::new().into()))
                })?;

                // Ensure that the motive matches the expected type
                let mut subst_ctx = ctx.levels().append(&vec![1].into());
                let on_exp_shifted = on_exp.shift((1, 0));
                let subst = Assign(Lvl { fst: subst_ctx.len() - 1, snd: 0 }, on_exp_shifted);
                let motive_t = ret_typ.subst(&mut subst_ctx, &subst).shift((-1, 0));
                let motive_t_nf = motive_t.normalize(prg, &mut ctx.env())?;
                convert(subst_ctx, motive_t_nf, &t)?;

                body_t =
                    ctx.bind_single(&self_binder, |ctx| ret_typ.normalize(prg, &mut ctx.env()))?;
                motive_out = Some(Motive {
                    span: *info,
                    param: ParamInst {
                        span: *info,
                        info: Some(self_t_nf),
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

        Ok(LocalMatch {
            span: *span,
            ctx: Some(ctx.vars.clone()),
            name: name.clone(),
            on_exp: on_exp_out,
            motive: motive_out,
            ret_typ: ret_typ_out.into(),
            body: body_out,
            inferred_type: Some(typ_app),
        })
    }
}

impl Check for LocalComatch {
    type Target = LocalComatch;

    fn check(&self, prg: &Prg, ctx: &mut Ctx, t: Rc<Exp>) -> Result<Self::Target, TypeError> {
        let LocalComatch { span, name, is_lambda_sugar, body, .. } = self;
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

        Ok(LocalComatch {
            span: *span,
            ctx: Some(ctx.vars.clone()),
            name: name.clone(),
            is_lambda_sugar: *is_lambda_sugar,
            body: body_out,
            inferred_type: Some(typ_app),
        })
    }
}

impl Check for Hole {
    type Target = Hole;

    fn check(&self, _prg: &Prg, ctx: &mut Ctx, t: Rc<Exp>) -> Result<Self::Target, TypeError> {
        let Hole { span, .. } = self;
        Ok(Hole {
            span: *span,
            inferred_type: Some(t.clone()),
            inferred_ctx: Some(ctx.vars.clone()),
        })
    }
}

/// Infer an expression
impl Infer for Exp {
    type Target = Exp;

    #[trace("{:P} |- {:P} => {return:P}", ctx, self, |ret| ret.as_ref().map(|e| e.typ()))]
    fn infer(&self, prg: &Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        match self {
            Exp::Variable(e) => Ok(Exp::Variable(e.infer(prg, ctx)?)),
            Exp::TypCtor(e) => Ok(Exp::TypCtor(e.infer(prg, ctx)?)),
            Exp::Call(e) => Ok(Exp::Call(e.infer(prg, ctx)?)),
            Exp::DotCall(e) => Ok(Exp::DotCall(e.infer(prg, ctx)?)),
            Exp::Anno(e) => Ok(Exp::Anno(e.infer(prg, ctx)?)),
            Exp::TypeUniv(e) => Ok(Exp::TypeUniv(e.infer(prg, ctx)?)),
            Exp::Hole(e) => Ok(Exp::Hole(e.infer(prg, ctx)?)),
            Exp::LocalMatch(e) => Ok(Exp::LocalMatch(e.infer(prg, ctx)?)),
            Exp::LocalComatch(e) => Ok(Exp::LocalComatch(e.infer(prg, ctx)?)),
        }
    }
}

impl Infer for Variable {
    type Target = Variable;

    fn infer(&self, _prg: &Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let Variable { span, idx, name, .. } = self;
        let typ_nf = ctx.lookup(*idx);
        Ok(Variable { span: *span, idx: *idx, name: name.clone(), inferred_type: Some(typ_nf) })
    }
}

impl Infer for TypCtor {
    type Target = TypCtor;

    fn infer(&self, prg: &Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let TypCtor { span, name, args } = self;
        let TypAbs { params } = &*prg.decls.typ(name, *span)?.typ();
        let args_out = check_args(args, prg, name, ctx, params, *span)?;

        Ok(TypCtor { span: *span, name: name.clone(), args: args_out })
    }
}

impl Infer for Call {
    type Target = Call;

    fn infer(&self, prg: &Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let Call { span, name, args, .. } = self;
        let Ctor { name, params, typ, .. } = &prg.decls.ctor_or_codef(name, *span)?;
        let args_out = check_args(args, prg, name, ctx, params, *span)?;
        let typ_out =
            typ.subst_under_ctx(vec![params.len()].into(), &vec![args.args.clone()]).to_exp();
        let typ_nf = typ_out.normalize(prg, &mut ctx.env())?;
        Ok(Call { span: *span, name: name.clone(), args: args_out, inferred_type: Some(typ_nf) })
    }
}

impl Infer for DotCall {
    type Target = DotCall;

    fn infer(&self, prg: &Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let DotCall { span, exp, name, args, .. } = self;
        let Dtor { name, params, self_param, ret_typ, .. } = &prg.decls.dtor_or_def(name, *span)?;

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

        Ok(DotCall {
            span: *span,
            exp: exp_out,
            name: name.clone(),
            args: args_out,
            inferred_type: Some(typ_out_nf),
        })
    }
}

impl Infer for Anno {
    type Target = Anno;

    fn infer(&self, prg: &Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let Anno { span, exp, typ, .. } = self;
        let typ_out = typ.check(prg, ctx, Rc::new(TypeUniv::new().into()))?;
        let typ_nf = typ.normalize(prg, &mut ctx.env())?;
        let exp_out = (**exp).check(prg, ctx, typ_nf.clone())?;
        Ok(Anno { span: *span, exp: Rc::new(exp_out), typ: typ_out, normalized_type: Some(typ_nf) })
    }
}

impl Infer for TypeUniv {
    type Target = TypeUniv;

    fn infer(&self, _prg: &Prg, _ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        Ok(self.clone())
    }
}

impl Infer for Hole {
    type Target = Hole;

    fn infer(&self, _prg: &Prg, _ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let Hole { span, .. } = self;
        Ok(Hole {
            span: *span,
            inferred_type: Some(Rc::new(Hole::new().into())),
            inferred_ctx: None,
        })
    }
}

impl Infer for LocalMatch {
    type Target = LocalMatch;

    fn infer(&self, _prg: &Prg, _ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        Err(TypeError::CannotInferMatch { span: self.span().to_miette() })
    }
}

impl Infer for LocalComatch {
    type Target = LocalComatch;

    fn infer(&self, _prg: &Prg, _ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        Err(TypeError::CannotInferComatch { span: self.span().to_miette() })
    }
}

fn check_args(
    this: &Args,
    prg: &Prg,
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

impl CheckTelescope for TelescopeInst {
    type Target = TelescopeInst;

    fn check_telescope<T, F: FnOnce(&mut Ctx, Self::Target) -> Result<T, TypeError>>(
        &self,
        prg: &Prg,
        name: &str,
        ctx: &mut Ctx,
        param_types: &Telescope,
        f: F,
        span: Option<Span>,
    ) -> Result<T, TypeError> {
        let Telescope { params: param_types } = param_types;
        let TelescopeInst { params } = self;

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
                let ParamInst { span, name, .. } = param_actual;
                let Param { typ, .. } = param_expected;
                let typ_out = typ.check(prg, ctx, Rc::new(TypeUniv::new().into()))?;
                let typ_nf = typ.normalize(prg, &mut ctx.env())?;
                let mut params_out = params_out;
                let param_out = ParamInst {
                    span: *span,
                    info: Some(typ_nf.clone()),
                    name: name.clone(),
                    typ: typ_out.into(),
                };
                params_out.push(param_out);
                let elem = Binder { name: param_actual.name.clone(), typ: typ_nf };
                Result::<_, TypeError>::Ok(BindElem { elem, ret: params_out })
            },
            |ctx, params| f(ctx, TelescopeInst { params }),
        )?
    }
}

impl InferTelescope for Telescope {
    type Target = Telescope;

    fn infer_telescope<T, F: FnOnce(&mut Ctx, Self::Target) -> Result<T, TypeError>>(
        &self,
        prg: &Prg,
        ctx: &mut Ctx,
        f: F,
    ) -> Result<T, TypeError> {
        let Telescope { params } = self;

        ctx.bind_fold_failable(
            params.iter(),
            vec![],
            |ctx, mut params_out, param| {
                let Param { typ, name } = param;
                let typ_out = typ.check(prg, ctx, Rc::new(TypeUniv::new().into()))?;
                let typ_nf = typ.normalize(prg, &mut ctx.env())?;
                let param_out = Param { name: name.clone(), typ: typ_out };
                params_out.push(param_out);
                let elem = Binder { name: param.name.clone(), typ: typ_nf };
                Result::<_, TypeError>::Ok(BindElem { elem, ret: params_out })
            },
            |ctx, params| f(ctx, Telescope { params }),
        )?
    }
}

impl InferTelescope for SelfParam {
    type Target = SelfParam;

    fn infer_telescope<T, F: FnOnce(&mut Ctx, Self::Target) -> Result<T, TypeError>>(
        &self,
        prg: &Prg,
        ctx: &mut Ctx,
        f: F,
    ) -> Result<T, TypeError> {
        let SelfParam { info, name, typ } = self;

        let typ_nf = typ.to_exp().normalize(prg, &mut ctx.env())?;
        let typ_out = typ.infer(prg, ctx)?;
        let param_out = SelfParam { info: *info, name: name.clone(), typ: typ_out };
        let elem = Binder { name: name.clone().unwrap_or_default(), typ: typ_nf };

        // We need to shift the self parameter type here because we treat it as a 1-element telescope
        ctx.bind_single(&elem.shift((1, 0)), |ctx| f(ctx, param_out))
    }
}

impl<T: Check> Check for Rc<T> {
    type Target = Rc<T::Target>;

    fn check(&self, prg: &Prg, ctx: &mut Ctx, t: Rc<Exp>) -> Result<Self::Target, TypeError> {
        Ok(Rc::new((**self).check(prg, ctx, t)?))
    }
}

impl<T: Infer> Infer for Rc<T> {
    type Target = Rc<T::Target>;

    fn infer(&self, prg: &Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        Ok(Rc::new((**self).infer(prg, ctx)?))
    }
}
