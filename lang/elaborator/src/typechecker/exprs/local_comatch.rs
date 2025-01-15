//! Bidirectional type checker

use crate::index_unification::constraints::Constraint;
use crate::index_unification::unify::*;
use crate::normalizer::env::ToEnv;
use crate::normalizer::normalize::Normalize;
use crate::typechecker::exprs::CheckTelescope;
use crate::typechecker::type_info_table::DtorMeta;
use ast::ctx::LevelCtx;
use ast::*;
use miette_util::ToMiette;

use super::super::ctx::*;
use super::super::util::*;
use super::CheckInfer;
use crate::result::TypeError;

impl CheckInfer for LocalComatch {
    fn check(&self, ctx: &mut Ctx, t: &Exp) -> Result<Self, TypeError> {
        let LocalComatch { span, name, is_lambda_sugar, cases, .. } = self;
        // The expected type that we check against should be a type constructor applied to
        // arguments.
        let expected_type_app: TypCtor = t.expect_typ_app()?.infer(ctx)?;

        // Local comatches don't support self parameters, yet.
        let codata = ctx.type_info_table.lookup_codata(&expected_type_app.name)?;
        if uses_self(codata)? {
            return Err(TypeError::LocalComatchWithSelf {
                type_name: codata.name.to_owned().id,
                span: span.to_miette(),
            });
        }

        let with_expected_type =
            WithExpectedType { cases, label: None, expected_type: expected_type_app.clone() };

        with_expected_type.check_exhaustiveness(ctx)?;
        let cases = with_expected_type.check_type(ctx)?;

        Ok(LocalComatch {
            span: *span,
            ctx: Some(ctx.vars.clone()),
            name: name.clone(),
            is_lambda_sugar: *is_lambda_sugar,
            cases,
            inferred_type: Some(expected_type_app),
        })
    }

    fn infer(&self, __ctx: &mut Ctx) -> Result<Self, TypeError> {
        // We cannot currently infer the type of a copattern match, only check against an expected type.
        Err(TypeError::CannotInferComatch { span: self.span().to_miette() })
    }
}

/// This struct is used to share code between the typechecking of local and global comatches.
pub struct WithExpectedType<'a> {
    pub cases: &'a Vec<Case>,
    /// Name of the global codefinition that gets substituted for the destructor's self parameters
    /// This is `None` for a local comatch.
    pub label: Option<(IdBound, usize)>,
    /// The expected type of the comatch, i.e. `Stream(Int)` for `comatch { hd => 1, tl => ... }`.
    pub expected_type: TypCtor,
}

/// Infer a copattern match
impl WithExpectedType<'_> {
    /// Check whether the copattern match contains exactly one clause for every
    /// destructor declared in the codata type declaration.
    pub fn check_exhaustiveness(&self, ctx: &mut Ctx) -> Result<(), TypeError> {
        let WithExpectedType { cases, .. } = &self;
        // Check that this comatch is on a codata type
        let codata = ctx.type_info_table.lookup_codata(&self.expected_type.name)?;

        // Check exhaustiveness
        let dtors_expected: HashSet<_> =
            codata.dtors.iter().map(|dtor| dtor.name.to_owned()).collect();
        let mut dtors_actual = HashSet::<IdBind>::default();
        let mut dtors_duplicate = HashSet::<IdBind>::default();

        for name in cases.iter().map(|case| &case.pattern.name) {
            if dtors_actual.contains(&name.clone().into()) {
                dtors_duplicate.insert(name.clone().into());
            }
            dtors_actual.insert(name.clone().into());
        }

        let mut dtors_missing = dtors_expected.difference(&dtors_actual).peekable();
        let mut dtors_exessive = dtors_actual.difference(&dtors_expected).peekable();

        if (dtors_missing.peek().is_some())
            || dtors_exessive.peek().is_some()
            || !dtors_duplicate.is_empty()
        {
            return Err(TypeError::invalid_match(
                dtors_missing.map(|i| &i.id).cloned().collect(),
                dtors_exessive.map(|i| &i.id).cloned().collect(),
                dtors_duplicate.into_iter().map(|i| i.id).collect(),
                &self.expected_type.span(),
            ));
        }
        Ok(())
    }

    /// Type-check the comatch
    pub fn check_type(&self, ctx: &mut Ctx) -> Result<Vec<Case>, TypeError> {
        let WithExpectedType { cases, expected_type, label } = &self;
        let TypCtor { args: on_args, .. } = expected_type;

        // We will compare `on_args` against `def_args`. But `def_args` are defined
        // in the context `params`. Below, we extend the context using `params_inst.check_telescope(...)`.
        // In this extended context we have to weaken on_args by shifting.
        let on_args = shift_and_clone(on_args, (1, 0));

        let mut cases_out = Vec::new();

        for case in cases.iter().cloned() {
            let DtorMeta { self_param, ret_typ, params, .. } =
                ctx.type_info_table.lookup_dtor(&case.pattern.name)?;
            let SelfParam { typ: TypCtor { args: def_args, .. }, .. } = self_param;
            let Case {
                span,
                pattern: Pattern { span: pattern_span, name, params: params_inst, .. },
                body,
            } = &case;
            // We are in the following situation:
            //
            // codata T(...) {  (self : T(.....)).d(...) : t, ...}
            //                            ^^^^^   ^ ^^^    ^
            //                              |     |  |     \------ ret_typ
            //                              |     |  \------------ params
            //                              |     \--------------- name
            //                              \--------------------- def_args
            //
            // comatch { d(...) => e, ...}
            //           ^ ^^^     ^
            //           |  |      \------------------------------ body
            //           |  \------------------------------------- params_inst
            //           \---------------------------------------- name
            //
            // where T(...) is the expected type of the comatch.
            //         ^^^
            //          \----------------------------------------- on_args

            // Normalize the arguments of the return type and the arguments to the self-parameter
            // of the destructor declaration.
            // TODO: Why can't we do this once *before* we repeatedly look them up in the context?
            let def_args = def_args
                .normalize(&ctx.type_info_table, &mut LevelCtx::from(vec![params.len()]).env())?;
            let ret_typ = ret_typ.normalize(
                &ctx.type_info_table,
                &mut LevelCtx::from(vec![params.len(), 1]).env(),
            )?;

            let name = name.clone();
            let params = params.clone();

            params_inst.check_telescope(
                &name.id,
                ctx,
                &params,
                |ctx, args_out| {
                    // We have to check whether we have an absurd case or an ordinary case.
                    // To do this we have solve the following unification problem:
                    //
                    //               T(...) =? T(...)
                    //                 ^^^       ^^^
                    //                  |         \----------------------- on_args
                    //                  \--------------------------------- def_args
                    //
                    let constraint = Constraint::EqualityArgs {
                        lhs: Args { args: def_args },
                        rhs: on_args.clone(),
                    };

                    match body {
                        // The programmer wrote an absurd case. We therefore have to check whether
                        // this case is really absurd. To do this, we verify that the unification
                        // actually fails.
                        None => {
                            unify(ctx.levels(), constraint, span)?
                                .map_yes(|_| TypeError::PatternIsNotAbsurd {
                                    name: Box::new(name.clone()),
                                    span: span.to_miette(),
                                })
                                .ok_no()?;

                            let case_out = Case {
                                span: *span,
                                pattern: Pattern {
                                    span: *pattern_span,
                                    is_copattern: true,
                                    name: name.clone(),
                                    params: args_out,
                                },
                                body: None,
                            };

                            cases_out.push(case_out);

                            Ok(())
                        }
                        Some(body) => {
                            // The programmer wrote a non-absurd case. We therefore have to check
                            // that the unification succeeds.

                            // We compute the return type for that specific cocase.
                            // E.g. for the following comatch:
                            // ```text
                            // codef Ones : Stream(Nat) {
                            //    hd => 1
                            //    tl => Ones
                            // }
                            // ```
                            // we compute the types `Nat` resp, `Stream(Nat)` for the respective
                            // cocases.
                            let ret_typ_nf = match label {
                                Some((label, n_label_args)) => {
                                    // We know that we are checking a *global* comatch which can use
                                    // the self parameter in its return type.
                                    // The term that we have to substitute for `self` is:
                                    // ```text
                                    // C(x, ... x_n)
                                    // ^          ^
                                    // |          \---- n_label_args
                                    // \--------------- label
                                    //
                                    // ```
                                    let args = (0..*n_label_args)
                                        .rev()
                                        .map(|snd| {
                                            Arg::UnnamedArg {
                                                arg: Box::new(Exp::Variable(Variable {
                                                    span: None,
                                                    // The field `fst` has to be `2` because we have two surrounding telescopes:
                                                    // - The arguments to the toplevel codefinition
                                                    // - The arguments bound by the destructor copattern.
                                                    idx: Idx { fst: 2, snd },
                                                    name: VarBound::from_string(""),
                                                    inferred_type: None,
                                                })),
                                                erased: false,
                                            }
                                        })
                                        .collect();
                                    let ctor = Box::new(Exp::Call(Call {
                                        span: None,
                                        kind: CallKind::Codefinition,
                                        name: label.clone(),
                                        args: Args { args },
                                        inferred_type: None,
                                    }));

                                    // Recall that we are in the following situation:
                                    //
                                    // codata T(...) {  (self : T(  σ  )).d( Ξ ) : t, ...}
                                    //                            ^^^^^   ^ ^^^    ^
                                    //                              |     |  |     \------ ret_typ
                                    //                              |     |  \------------ params
                                    //                              |     \--------------- name
                                    //                              \--------------------- def_args
                                    //
                                    // codef C(Δ) { d( Ξ ) => e, ...}
                                    //              ^ ^^^     ^
                                    //              |  |      \------------------------------ body
                                    //              |  \------------------------------------- params_inst
                                    //              \---------------------------------------- name
                                    //
                                    // Note that t is typed under the following context:
                                    // Ξ;self |- t : Type
                                    // We want to perform the following substitution:
                                    // Δ;Ξ |- [C id_Δ / self]t : Type
                                    // To represent id_Δ, we mentally extend the context by self as follows:
                                    // Δ;Ξ;self |- C id_Δ : Type
                                    // This is why id_Δ = [(2,n), (2, n-1), ..., (2, 0)]
                                    // Since t is defined under context Ξ;self, we
                                    // still substitute for level (1, 0) which corresponds to self under context Ξ;self.
                                    // The result is under context Δ;Ξ;self, which we shift by (-1, 0) to get rid of self
                                    // (which no longer occurs) in [C id_Δ / self]t.
                                    // So we finally have:
                                    // Δ;Ξ |- [C id_Δ / self]t : Type
                                    //
                                    let subst = Assign { lvl: Lvl { fst: 1, snd: 0 }, exp: ctor };
                                    let mut subst_ctx = LevelCtx::from(vec![params.len(), 1]);
                                    let mut ret_typ = ret_typ.subst(&mut subst_ctx, &subst);
                                    ret_typ.shift((-1, 0));
                                    ret_typ.normalize(
                                        &ctx.type_info_table,
                                        &mut LevelCtx::from(vec![*n_label_args, params.len()])
                                            .env(),
                                    )?
                                }

                                None => {
                                    // TODO: Self parameter for local comatches
                                    shift_and_clone(&ret_typ, (-1, 0))
                                }
                            };
                            let body_out = {
                                let unif = unify(ctx.levels(), constraint, span)?
                                    .map_no(|()| TypeError::PatternIsAbsurd {
                                        name: Box::new(name.clone()),
                                        span: span.to_miette(),
                                    })
                                    .ok_yes()?;

                                ctx.fork::<Result<_, TypeError>, _>(|ctx| {
                                    let type_info_table = ctx.type_info_table.clone();
                                    ctx.subst(&type_info_table, &unif)?;
                                    let body = body.subst(&mut ctx.levels(), &unif);

                                    let t_subst = ret_typ_nf.subst(&mut ctx.levels(), &unif);
                                    let t_nf =
                                        t_subst.normalize(&ctx.type_info_table, &mut ctx.env())?;

                                    let body_out = body.check(ctx, &t_nf)?;

                                    Ok(Some(body_out))
                                })?
                            };

                            let case_out = Case {
                                span: *span,
                                pattern: Pattern {
                                    span: *pattern_span,
                                    is_copattern: true,
                                    name: name.clone(),
                                    params: args_out,
                                },
                                body: body_out,
                            };

                            cases_out.push(case_out);

                            Ok(())
                        }
                    }
                },
                *span,
            )?;
        }

        Ok(cases_out)
    }
}
