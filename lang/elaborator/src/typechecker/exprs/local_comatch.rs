//! Bidirectional type checker

use std::rc::Rc;

use log::trace;

use printer::PrintToString;

use crate::normalizer::env::ToEnv;
use crate::normalizer::normalize::Normalize;
use crate::typechecker::exprs::CheckTelescope;
use crate::unifier::unify::*;
use miette_util::ToMiette;
use syntax::ast::*;
use syntax::common::*;
use syntax::ctx::LevelCtx;

use super::super::ctx::*;
use super::super::util::*;
use super::CheckInfer;
use crate::result::TypeError;

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

/// Infer a cocase in a co-pattern match
fn check_cocase(
    eqns: Vec<Eqn>,
    cocase: &Case,
    prg: &Module,
    ctx: &mut Ctx,
    t: Rc<Exp>,
) -> Result<Case, TypeError> {
    trace!(
        "{} |- {} <= {}",
        ctx.print_to_colored_string(None),
        cocase.print_to_colored_string(None),
        t.print_to_colored_string(None)
    );
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
