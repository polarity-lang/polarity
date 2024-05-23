//! Bidirectional type checker

use std::rc::Rc;

use codespan::Span;

use crate::normalizer::env::ToEnv;
use crate::normalizer::normalize::Normalize;
use crate::unifier::unify::*;
use miette_util::ToMiette;
use syntax::ast::*;
use syntax::common::*;
use syntax::ctx::values::Binder;
use syntax::ctx::{BindContext, BindElem, LevelCtx};
use tracer::trace;

use super::ctx::*;
use super::util::*;
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
    #[trace("{:P} |- {:P} <= {:P}", ctx, self, t)]
    fn check(&self, prg: &Module, ctx: &mut Ctx, t: Rc<Exp>) -> Result<Self, TypeError> {
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

    #[trace("{:P} |- {:P} => {return:P}", ctx, self, |ret| ret.as_ref().map(|e| e.typ()))]
    fn infer(&self, prg: &Module, ctx: &mut Ctx) -> Result<Self, TypeError> {
        match self {
            Exp::Variable(e) => Ok(e.infer(prg, ctx)?.into()),
            Exp::TypCtor(e) => Ok(e.infer(prg, ctx)?.into()),
            Exp::Call(e) => Ok(e.infer(prg, ctx)?.into()),
            Exp::DotCall(e) => Ok(e.infer(prg, ctx)?.into()),
            Exp::Anno(e) => Ok(e.infer(prg, ctx)?.into()),
            Exp::TypeUniv(e) => Ok(e.infer(prg, ctx)?.into()),
            Exp::Hole(e) => Ok(e.infer(prg, ctx)?.into()),
            Exp::LocalMatch(e) => Ok(e.infer(prg, ctx)?.into()),
            Exp::LocalComatch(e) => Ok(e.infer(prg, ctx)?.into()),
        }
    }
}

// Variable
//
//

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
        Ok(Variable { span: *span, idx: *idx, name: name.clone(), inferred_type: Some(typ_nf) })
    }
}

// TypCtor
//
//

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

// Call
//
//

impl CheckInfer for Call {
    /// The *checking* rule for calls is:
    /// ```text
    ///                 ...
    ///           ──────────────────
    ///            P, Γ ⊢ Cσ ⇐ τ
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
    /// The *inference* rule for calls is:
    /// ```text
    ///                 ...
    ///           ──────────────────
    ///            P, Γ ⊢ Cσ ⇒ ...
    /// ```
    fn infer(&self, prg: &Module, ctx: &mut Ctx) -> Result<Self, TypeError> {
        let Call { span, kind, name, args, .. } = self;

        match kind {
            CallKind::Codefinition | CallKind::Constructor => {
                let Ctor { name, params, typ, .. } = &prg.ctor_or_codef(name, *span)?;
                let args_out = check_args(args, prg, name, ctx, params, *span)?;
                let typ_out = typ
                    .subst_under_ctx(vec![params.len()].into(), &vec![args.args.clone()])
                    .to_exp();
                let typ_nf = typ_out.normalize(prg, &mut ctx.env())?;
                Ok(Call {
                    span: *span,
                    kind: *kind,
                    name: name.clone(),
                    args: args_out,
                    inferred_type: Some(typ_nf),
                })
            }
            CallKind::LetBound => {
                let Let { name, params, typ, .. } = prg.top_level_let(name, *span)?;
                let args_out = check_args(args, prg, name, ctx, params, *span)?;
                let typ_out =
                    typ.subst_under_ctx(vec![params.len()].into(), &vec![args.args.clone()]);
                let typ_nf = typ_out.normalize(prg, &mut ctx.env())?;
                Ok(Call {
                    span: *span,
                    kind: *kind,
                    name: name.clone(),
                    args: args_out,
                    inferred_type: Some(typ_nf),
                })
            }
        }
    }
}

// DotCall
//
//

impl CheckInfer for DotCall {
    /// The *checking* rule for dotcalls is:
    /// ```text
    ///                 ...
    ///           ──────────────────
    ///            P, Γ ⊢ e.Dσ ⇐ τ
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

    /// The *inference* rule for dotcalls is:
    /// ```text
    ///                 ...
    ///           ──────────────────
    ///            P, Γ ⊢ e.Dσ ⇒ ...
    /// ```
    fn infer(&self, prg: &Module, ctx: &mut Ctx) -> Result<Self, TypeError> {
        let DotCall { span, kind, exp, name, args, .. } = self;
        let Dtor { name, params, self_param, ret_typ, .. } = &prg.dtor_or_def(name, *span)?;

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
            kind: *kind,
            exp: exp_out,
            name: name.clone(),
            args: args_out,
            inferred_type: Some(typ_out_nf),
        })
    }
}

// Anno
//
//

impl CheckInfer for Anno {
    fn check(&self, prg: &Module, ctx: &mut Ctx, t: Rc<Exp>) -> Result<Self, TypeError> {
        let inferred_term = self.infer(prg, ctx)?;
        let inferred_typ = inferred_term.typ().ok_or(TypeError::Impossible {
            message: "Expected inferred type".to_owned(),
            span: None,
        })?;
        convert(ctx.levels(), &mut ctx.meta_vars, inferred_typ, &t)?;
        Ok(inferred_term)
    }

    /// The *inference* rule for type annotations is:
    /// ```text
    ///            P, Γ ⊢ τ ⇐ Type
    ///            P, Γ ⊢ τ ▷ τ'
    ///            P, Γ ⊢ e ⇐ τ'
    ///           ──────────────────────
    ///            P, Γ ⊢ (e : τ) ⇒ τ'
    /// ```
    fn infer(&self, prg: &Module, ctx: &mut Ctx) -> Result<Self, TypeError> {
        let Anno { span, exp, typ, .. } = self;
        let typ_out = typ.check(prg, ctx, Rc::new(TypeUniv::new().into()))?;
        let typ_nf = typ.normalize(prg, &mut ctx.env())?;
        let exp_out = (**exp).check(prg, ctx, typ_nf.clone())?;
        Ok(Anno { span: *span, exp: Rc::new(exp_out), typ: typ_out, normalized_type: Some(typ_nf) })
    }
}

// TypeUniv
//
//

impl CheckInfer for TypeUniv {
    /// The *checking* rule for the type universe is:
    /// ```text
    ///            P, Γ ⊢ τ ≃ Type
    ///           ──────────────────
    ///            P, Γ ⊢ Type ⇐ τ
    /// ```
    fn check(&self, _prg: &Module, ctx: &mut Ctx, t: Rc<Exp>) -> Result<Self, TypeError> {
        convert(ctx.levels(), &mut ctx.meta_vars, Rc::new(TypeUniv::new().into()), &t)?;
        Ok(self.clone())
    }

    /// The *inference* rule for the type universe is:
    /// ```text
    ///           ─────────────────────
    ///            P, Γ ⊢ Type ⇒ Type
    /// ```
    /// Note: The type universe is impredicative and the theory
    /// therefore inconsistent.
    fn infer(&self, _prg: &Module, _ctx: &mut Ctx) -> Result<Self, TypeError> {
        Ok(self.clone())
    }
}

// Hole
//
//

impl CheckInfer for Hole {
    fn check(&self, prg: &Module, ctx: &mut Ctx, t: Rc<Exp>) -> Result<Self, TypeError> {
        let Hole { span, metavar, args, .. } = self;
        let args: Vec<Vec<Rc<Exp>>> = args
            .iter()
            .map(|subst| subst.iter().map(|exp| exp.infer(prg, ctx)).collect::<Result<Vec<_>, _>>())
            .collect::<Result<_, _>>()?;
        Ok(Hole {
            span: *span,
            metavar: *metavar,
            inferred_type: Some(t.clone()),
            inferred_ctx: Some(ctx.vars.clone()),
            args,
        })
    }

    fn infer(&self, _prg: &Module, _ctx: &mut Ctx) -> Result<Self, TypeError> {
        Err(TypeError::CannotInferHole { span: self.span().to_miette() })
    }
}

// LocalMatch
//
//

impl CheckInfer for LocalMatch {
    fn check(&self, prg: &Module, ctx: &mut Ctx, t: Rc<Exp>) -> Result<Self, TypeError> {
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
                let self_t_nf = typ_app.to_exp().normalize(prg, &mut ctx.env())?.shift((1, 0));
                let self_binder = Binder { name: param.name.clone(), typ: self_t_nf.clone() };

                // Typecheck the motive
                let ret_typ_out = ctx.bind_single(&self_binder, |ctx| {
                    ret_typ.check(prg, ctx, Rc::new(TypeUniv::new().into()))
                })?;

                // Ensure that the motive matches the expected type
                let mut subst_ctx = ctx.levels().append(&vec![1].into());
                let on_exp_shifted = on_exp.shift((1, 0));
                let subst =
                    Assign { lvl: Lvl { fst: subst_ctx.len() - 1, snd: 0 }, exp: on_exp_shifted };
                let motive_t = ret_typ.subst(&mut subst_ctx, &subst).shift((-1, 0));
                let motive_t_nf = motive_t.normalize(prg, &mut ctx.env())?;
                convert(subst_ctx, &mut ctx.meta_vars, motive_t_nf, &t)?;

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

        let body_out = WithScrutinee { inner: body, scrutinee: typ_app_nf.clone() }
            .check_ws(prg, ctx, body_t)?;

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

    fn infer(&self, _prg: &Module, _ctx: &mut Ctx) -> Result<Self, TypeError> {
        Err(TypeError::CannotInferMatch { span: self.span().to_miette() })
    }
}

// LocalComatch
//
//

impl CheckInfer for LocalComatch {
    fn check(&self, prg: &Module, ctx: &mut Ctx, t: Rc<Exp>) -> Result<Self, TypeError> {
        let LocalComatch { span, name, is_lambda_sugar, body, .. } = self;
        let typ_app_nf = t.expect_typ_app()?;
        let typ_app = typ_app_nf.infer(prg, ctx)?;

        // Local comatches don't support self parameters, yet.
        let codata = prg.codata(&typ_app.name, *span)?;
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
        let body_out = wd.infer_wd(prg, ctx)?;

        Ok(LocalComatch {
            span: *span,
            ctx: Some(ctx.vars.clone()),
            name: name.clone(),
            is_lambda_sugar: *is_lambda_sugar,
            body: body_out,
            inferred_type: Some(typ_app),
        })
    }

    fn infer(&self, _prg: &Module, _ctx: &mut Ctx) -> Result<Self, TypeError> {
        Err(TypeError::CannotInferComatch { span: self.span().to_miette() })
    }
}

// Other
//
//

pub trait CheckTelescope {
    type Target;

    fn check_telescope<T, F: FnOnce(&mut Ctx, Self::Target) -> Result<T, TypeError>>(
        &self,
        prg: &Module,
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
        prg: &Module,
        ctx: &mut Ctx,
        f: F,
    ) -> Result<T, TypeError>;
}

pub struct WithScrutinee<'a> {
    pub inner: &'a Match,
    pub scrutinee: TypCtor,
}

/// Check a pattern match
impl<'a> WithScrutinee<'a> {
    pub fn check_ws(&self, prg: &Module, ctx: &mut Ctx, t: Rc<Exp>) -> Result<Match, TypeError> {
        let Match { span, cases, omit_absurd } = &self.inner;

        // Check that this match is on a data type
        let data = prg.data(&self.scrutinee.name, *span)?;

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
                let Ctor { params, .. } = prg.ctor(&name, *span)?;

                let case = Case { span: *span, name, params: params.instantiate(), body: None };
                cases.push((case, true));
            }
        }

        let mut cases_out = Vec::new();

        for (case, omit) in cases {
            // Build equations for this case
            let Ctor { typ: TypCtor { args: def_args, .. }, params, .. } =
                prg.ctor(&case.name, case.span)?;

            let def_args_nf = LevelCtx::empty()
                .bind_iter(params.params.iter(), |ctx| def_args.normalize(prg, &mut ctx.env()))?;

            let TypCtor { args: on_args, .. } = &self.scrutinee;
            let on_args = on_args.shift((1, 0)); // FIXME: where to shift this

            let eqns: Vec<_> = def_args_nf
                .iter()
                .cloned()
                .zip(on_args.args.iter().cloned())
                .map(|(lhs, rhs)| Eqn { lhs, rhs })
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

pub struct WithDestructee<'a> {
    pub inner: &'a Match,
    /// Name of the global codefinition that gets substituted for the destructor's self parameters
    pub label: Option<Ident>,
    pub n_label_args: usize,
    pub destructee: TypCtor,
}

/// Infer a copattern match
impl<'a> WithDestructee<'a> {
    pub fn infer_wd(&self, prg: &Module, ctx: &mut Ctx) -> Result<Match, TypeError> {
        let Match { span, cases, omit_absurd } = &self.inner;

        // Check that this comatch is on a codata type
        let codata = prg.codata(&self.destructee.name, *span)?;

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
                let Dtor { params, .. } = prg.dtor(&name, *span)?;

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
            } = prg.dtor(&case.name, case.span)?;

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
                .map(|(lhs, rhs)| Eqn { lhs, rhs })
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
                        kind: CallKind::Codefinition,
                        name: label.clone(),
                        args: Args { args },
                        inferred_type: None,
                    }));
                    let subst = Assign { lvl: Lvl { fst: 1, snd: 0 }, exp: ctor };
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
    prg: &Module,
    ctx: &mut Ctx,
    t: Rc<Exp>,
) -> Result<Case, TypeError> {
    let Case { span, name, params: args, body } = case;
    let Ctor { name, params, .. } = prg.ctor(name, *span)?;

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
                kind: CallKind::Constructor,
                name: name.clone(),
                args: Args { args },
                inferred_type: None,
            }));
            let subst = Assign { lvl: Lvl { fst: curr_lvl, snd: 0 }, exp: ctor };

            // FIXME: Refactor this
            let t = t
                .shift((1, 0))
                .swap_with_ctx(&mut subst_ctx_1, curr_lvl, curr_lvl - 1)
                .subst(&mut subst_ctx_2, &subst)
                .shift((-1, 0));

            let body_out = match body {
                Some(body) => {
                    let unif = unify(ctx.levels(), &mut ctx.meta_vars, eqns.clone(), false)?
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
                    unify(ctx.levels(), &mut ctx.meta_vars, eqns.clone(), false)?
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
    prg: &Module,
    ctx: &mut Ctx,
    t: Rc<Exp>,
) -> Result<Case, TypeError> {
    let Case { span, name, params: params_inst, body } = cocase;
    let Dtor { name, params, .. } = prg.dtor(name, *span)?;

    params_inst.check_telescope(
        prg,
        name,
        ctx,
        params,
        |ctx, args_out| {
            let body_out = match body {
                Some(body) => {
                    let unif = unify(ctx.levels(), &mut ctx.meta_vars, eqns.clone(), false)?
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
                    unify(ctx.levels(), &mut ctx.meta_vars, eqns.clone(), false)?
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

impl CheckTelescope for TelescopeInst {
    type Target = TelescopeInst;

    fn check_telescope<T, F: FnOnce(&mut Ctx, Self::Target) -> Result<T, TypeError>>(
        &self,
        prg: &Module,
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
        prg: &Module,
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
        prg: &Module,
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
