//! Bidirectional type checker

use std::rc::Rc;

use log::trace;

use printer::types::Print;

use crate::normalizer::env::ToEnv;
use crate::normalizer::normalize::Normalize;
use crate::typechecker::exprs::CheckTelescope;
use crate::unifier::unify::*;
use miette_util::ToMiette;
use syntax::ast::*;
use syntax::common::*;
use syntax::ctx::values::Binder;
use syntax::ctx::{BindContext, LevelCtx};

use super::super::ctx::*;
use super::super::util::*;
use super::CheckInfer;
use crate::result::TypeError;

// LocalMatch
//
//

impl CheckInfer for LocalMatch {
    fn check(&self, prg: &Module, ctx: &mut Ctx, t: Rc<Exp>) -> Result<Self, TypeError> {
        let LocalMatch { span, name, on_exp, motive, cases, omit_absurd, .. } = self;
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

        let (cases, omit_absurd) =
            WithScrutinee { cases, omit_absurd: *omit_absurd, scrutinee: typ_app_nf.clone() }
                .check_ws(prg, ctx, body_t)?;

        Ok(LocalMatch {
            span: *span,
            ctx: Some(ctx.vars.clone()),
            name: name.clone(),
            on_exp: on_exp_out,
            motive: motive_out,
            ret_typ: ret_typ_out.into(),
            cases,
            omit_absurd,
            inferred_type: Some(typ_app),
        })
    }

    fn infer(&self, _prg: &Module, _ctx: &mut Ctx) -> Result<Self, TypeError> {
        Err(TypeError::CannotInferMatch { span: self.span().to_miette() })
    }
}

pub struct WithScrutinee<'a> {
    pub cases: &'a Vec<Case>,
    pub omit_absurd: bool,
    pub scrutinee: TypCtor,
}

/// Check a pattern match
impl<'a> WithScrutinee<'a> {
    pub fn check_ws(
        &self,
        prg: &Module,
        ctx: &mut Ctx,
        t: Rc<Exp>,
    ) -> Result<(Vec<Case>, bool), TypeError> {
        let WithScrutinee { cases, omit_absurd, .. } = &self;

        // Check that this match is on a data type
        let data = prg.data(&self.scrutinee.name, self.scrutinee.span())?;

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
                &self.scrutinee.span(),
            ));
        }

        // Add absurd cases for all omitted constructors
        let mut cases: Vec<_> = cases.iter().cloned().map(|case| (case, false)).collect();

        if *omit_absurd {
            for name in ctors_missing.cloned() {
                let Ctor { params, .. } = prg.ctor(&name, self.scrutinee.span())?;

                let case = Case {
                    span: self.scrutinee.span(),
                    name,
                    params: params.instantiate(),
                    body: None,
                };
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

        Ok((cases_out, *omit_absurd))
    }
}

/// Infer a case in a pattern match
fn check_case(
    eqns: Vec<Eqn>,
    case: &Case,
    prg: &Module,
    ctx: &mut Ctx,
    t: Rc<Exp>,
) -> Result<Case, TypeError> {
    trace!(
        "{} |- {} <= {}",
        ctx.print_to_colored_string(None),
        case.print_to_colored_string(None),
        t.print_to_colored_string(None)
    );
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
