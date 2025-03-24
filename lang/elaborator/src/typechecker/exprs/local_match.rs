//! Bidirectional type checker

use std::collections::HashSet;

use ast::ctx::values::Binder;
use ast::ctx::{BindContext, LevelCtx};
use ast::*;
use miette_util::ToMiette;

use crate::conversion_checking::convert;
use crate::index_unification::constraints::Constraint;
use crate::index_unification::unify::*;
use crate::normalizer::env::ToEnv;
use crate::normalizer::normalize::Normalize;
use crate::result::{TcResult, TypeError};
use crate::typechecker::exprs::CheckTelescope;
use crate::typechecker::type_info_table::CtorMeta;

use super::super::ctx::*;
use super::super::util::*;
use super::{CheckInfer, ExpectType};

// LocalMatch
//
//

impl CheckInfer for LocalMatch {
    fn check(&self, ctx: &mut Ctx, t: &Exp) -> TcResult<Self> {
        let LocalMatch { span, name, on_exp, motive, cases, .. } = self;

        // Compute the type of the expression we are pattern matching on.
        // This should always be a type constructor for a data type.
        let on_exp_out = on_exp.infer(ctx)?;
        let on_exp_typ = on_exp_out.expect_typ()?.expect_typ_app()?;

        let motive_out;
        let body_t;

        match motive {
            // Pattern matching with motive
            Some(m) => {
                let Motive { span: info, param, ret_typ } = m;
                let mut self_t_nf =
                    on_exp_typ.to_exp().normalize(&ctx.type_info_table, &mut ctx.env())?;
                self_t_nf.shift((1, 0));
                let self_binder = Binder { name: param.name.clone(), content: self_t_nf.clone() };

                // Typecheck the motive
                let ret_typ_out = ctx.bind_single(self_binder.clone(), |ctx| {
                    ret_typ.check(ctx, &Box::new(TypeUniv::new().into()))
                })?;

                // Ensure that the motive matches the expected type
                let motive_binder = Binder { name: m.param.name.clone(), content: () };
                let mut subst_ctx = ctx.levels().append(&vec![vec![motive_binder]].into());
                let on_exp = shift_and_clone(on_exp, (1, 0));
                let subst = Assign { lvl: Lvl { fst: subst_ctx.len() - 1, snd: 0 }, exp: on_exp };
                let mut motive_t = ret_typ.subst(&mut subst_ctx, &subst)?;
                motive_t.shift((-1, 0));
                let motive_t_nf = motive_t.normalize(&ctx.type_info_table, &mut ctx.env())?;
                convert(&ctx.vars, &mut ctx.meta_vars, motive_t_nf, t, span)?;

                body_t = ctx.bind_single(self_binder.clone(), |ctx| {
                    ret_typ.normalize(&ctx.type_info_table, &mut ctx.env())
                })?;
                motive_out = Some(Motive {
                    span: *info,
                    param: ParamInst {
                        span: *info,
                        info: Some(self_t_nf),
                        name: param.name.clone(),
                        typ: Box::new(on_exp_typ.to_exp()).into(),
                        erased: param.erased,
                    },
                    ret_typ: ret_typ_out,
                });
            }
            // Pattern matching without motive
            None => {
                body_t = Box::new(shift_and_clone(t, (1, 0)));
                motive_out = None;
            }
        };

        let with_scrutinee_type = WithScrutineeType {
            cases,
            scrutinee_type: on_exp_typ.clone(),
            scrutinee_name: VarBind::Wildcard { span: None },
        };
        with_scrutinee_type.check_exhaustiveness(ctx)?;
        let cases = with_scrutinee_type.check_type(ctx, &body_t)?;

        Ok(LocalMatch {
            span: *span,
            ctx: Some(ctx.vars.clone()),
            name: name.clone(),
            on_exp: on_exp_out,
            motive: motive_out,
            ret_typ: Some(Box::new(t.clone())),
            cases,
            inferred_type: Some(on_exp_typ),
        })
    }

    fn infer(&self, __ctx: &mut Ctx) -> TcResult<Self> {
        Err(TypeError::CannotInferMatch { span: self.span().to_miette() }.into())
    }
}

pub struct WithScrutineeType<'a> {
    pub cases: &'a Vec<Case>,
    pub scrutinee_type: TypCtor,
    pub scrutinee_name: VarBind,
}

/// Check a pattern match
impl WithScrutineeType<'_> {
    /// Check whether the pattern match contains exactly one clause for every
    /// constructor declared in the data type declaration.
    pub fn check_exhaustiveness(&self, ctx: &mut Ctx) -> TcResult {
        let WithScrutineeType { cases, .. } = &self;
        // Check that this match is on a data type
        let data = ctx.type_info_table.lookup_data(&self.scrutinee_type.name)?;

        // Check exhaustiveness
        let ctors_expected: HashSet<_> =
            data.ctors.iter().map(|ctor| ctor.name.to_owned()).collect();
        let mut ctors_actual: HashSet<IdBind> = HashSet::default();
        let mut ctors_duplicate: HashSet<IdBind> = HashSet::default();

        for name in cases.iter().map(|case| &case.pattern.name) {
            if ctors_actual.contains(&name.clone().into()) {
                ctors_duplicate.insert(name.clone().into());
            }
            ctors_actual.insert(name.clone().into());
        }
        let mut ctors_missing = ctors_expected.difference(&ctors_actual).peekable();
        let mut ctors_undeclared = ctors_actual.difference(&ctors_expected).peekable();

        if (ctors_missing.peek().is_some())
            || ctors_undeclared.peek().is_some()
            || !ctors_duplicate.is_empty()
        {
            return Err(TypeError::invalid_match(
                ctors_missing.map(|i| &i.id).cloned().collect(),
                ctors_undeclared.map(|i| &i.id).cloned().collect(),
                ctors_duplicate.into_iter().map(|i| i.id).collect(),
                &self.scrutinee_type.span(),
            ));
        }
        Ok(())
    }

    /// Typecheck the pattern match cases
    pub fn check_type(&self, ctx: &mut Ctx, t: &Exp) -> TcResult<Vec<Case>> {
        let WithScrutineeType { cases, scrutinee_name, .. } = &self;

        let cases: Vec<_> = cases.to_vec();
        let mut cases_out = Vec::new();

        for case in cases {
            log::trace!("Checking case for constructor: {}", case.pattern.name.id);

            let Case {
                span,
                pattern: Pattern { name, params: args, span: pattern_span, .. },
                body,
            } = case;
            let CtorMeta { typ: TypCtor { args: def_args, .. }, params, .. } =
                ctx.type_info_table.lookup_ctor(&name)?;
            let TypCtor { args: on_args, .. } = &self.scrutinee_type;
            // We are in the following situation:
            //
            // data T(...) {  C(...): T(...), ...}
            //                ^ ^^^     ^^^^
            //                |  |        \-------------------------- def_args
            //                |  \----------------------------------- params
            //                \-------------------------------------- name
            //
            // (... : T(...)).match as self => t { C(...) => e, ...}
            //          ^^^                          ^^^     ^
            //           |                            |      \------- body
            //           |                            \-------------- args
            //           \------------------------------------------- on_args

            // Normalize the arguments of the constructor type.
            // They will later be unified with the arguments of the scrutinee type.
            let def_args_nf = LevelCtx::empty().bind_iter(params.params.iter(), |ctx_| {
                def_args.normalize(&ctx.type_info_table, &mut ctx_.env())
            })?;

            // To check each individual case, we need to substitute the constructor for the self parameter
            // in the return type of the match t.
            // Recall that we are in the following situation:
            //
            // (... : T(...)).match as self => t { C(Ξ) => e, ...}
            //
            // Initally, t is defined under the context [self: T(...)].
            // Checking the body of the case against t must happen under the context Ξ.
            // Hence, to substitute the constructor for the self parameter within the body of the case,
            // we will do the following:
            //
            // * Extend the context with the pattern arguments: [self: T(...), Ξ]
            // * Swap the levels such that t has context [Ξ, self: T(...)]
            // * Substitute C(Ξ) for self
            // * Shift t by one level such that we end up with the context Ξ
            let mut subst_ctx_1 = ctx.levels().append(
                &vec![
                    vec![scrutinee_name.clone()],
                    params.params.iter().map(|p| p.name.clone()).collect(),
                ]
                .into(),
            );
            let mut subst_ctx_2 = ctx.levels().append(
                &vec![
                    params.params.iter().map(|p| p.name.clone()).collect(),
                    vec![scrutinee_name.clone()],
                ]
                .into(),
            );
            let curr_lvl = subst_ctx_2.len() - 1;

            let name = name.clone();
            let params = params.clone();

            args.check_telescope(
                &name.id,
                ctx,
                &params,
                |ctx, args_out| {
                    // Substitute the constructor for the self parameter
                    //
                    //
                    let args = (0..params.len())
                        .rev()
                        .map(|snd| Arg::UnnamedArg {
                            arg: Box::new(Exp::Variable(Variable {
                                span: None,
                                idx: Idx { fst: 1, snd },
                                name: VarBound::from_string(""),
                                inferred_type: None,
                            })),
                            erased: false,
                        })
                        .collect();
                    let ctor = Box::new(Exp::Call(Call {
                        span: None,
                        kind: CallKind::Constructor,
                        name: name.clone(),
                        args: Args { args },
                        inferred_type: None,
                    }));
                    let subst = Assign { lvl: Lvl { fst: curr_lvl, snd: 0 }, exp: ctor };
                    let mut t = t.clone();
                    t.shift((1, 0));
                    let mut t = t
                        .swap_with_ctx(&mut subst_ctx_1, curr_lvl, curr_lvl - 1)
                        .subst(&mut subst_ctx_2, &subst)?;
                    t.shift((-1, 0));

                    // We have to check whether we have an absurd case or an ordinary case.
                    // To do this we have solve the following unification problem:
                    //
                    //               T(...) =? T(...)
                    //                 ^^^       ^^^
                    //                  |         \----------------------- on_args
                    //                  \--------------------------------- def_args
                    //
                    // Recall that while def_args depends on the parameters of the constructor,
                    // on_args does not. Hence, we need to shift on_args by one telescope level s.t.
                    // the lhs and rhs of the unification constraint have the same context.
                    let on_args = shift_and_clone(on_args, (1, 0));
                    let constraint =
                        Constraint::EqualityArgs { lhs: Args { args: def_args_nf }, rhs: on_args };

                    let body_out = match body {
                        Some(body) => {
                            // The programmer wrote a non-absurd case. We therefore have to check
                            // that the unification succeeds.
                            let res = unify(ctx.levels(), constraint, &span)?;
                            let unif = match res {
                                crate::index_unification::dec::Dec::Yes(unif) => unif,
                                crate::index_unification::dec::Dec::No => {
                                    // A right-hand side was provided in the clause, but unification fails.
                                    let err = TypeError::PatternIsAbsurd {
                                        name: Box::new(name.clone()),
                                        span: span.to_miette(),
                                    };
                                    return Err(err.into());
                                }
                            };

                            ctx.fork::<TcResult<_>, _>(|ctx| {
                                let type_info_table = ctx.type_info_table.clone();
                                ctx.subst(&type_info_table, &unif)?;
                                let body = body.subst(&mut ctx.levels(), &unif)?;

                                let t_subst = t.subst(&mut ctx.levels(), &unif)?;
                                let t_nf =
                                    t_subst.normalize(&ctx.type_info_table, &mut ctx.env())?;

                                let body_out = body.check(ctx, &t_nf)?;

                                Ok(Some(body_out))
                            })?
                        }
                        None => {
                            // The programmer wrote an absurd case. We therefore have to check whether
                            // this case is really absurd. To do this, we verify that the unification
                            // actually fails.
                            let res = unify(ctx.levels(), constraint, &span)?;
                            if let crate::index_unification::dec::Dec::Yes(_) = res {
                                // The case was annotated as absurd but index unification succeeds.
                                let err = TypeError::PatternIsNotAbsurd {
                                    name: Box::new(name.clone()),
                                    span: span.to_miette(),
                                };
                                return Err(err.into());
                            }
                            None
                        }
                    };
                    let case_out = Case {
                        span,
                        pattern: Pattern {
                            span: pattern_span,
                            is_copattern: false,
                            name: name.clone(),
                            params: args_out,
                        },
                        body: body_out,
                    };
                    cases_out.push(case_out);
                    Ok(())
                },
                span,
            )?;
        }

        Ok(cases_out)
    }
}
