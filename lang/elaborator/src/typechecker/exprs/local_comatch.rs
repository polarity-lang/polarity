//! Bidirectional type checker

use std::rc::Rc;

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
        let LocalComatch { span, name, is_lambda_sugar, cases, .. } = self;
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

        let wd =
            WithDestructee { cases, label: None, n_label_args: 0, destructee: typ_app_nf.clone() };

        wd.check_exhaustiveness(prg)?;
        let cases = wd.infer_wd(prg, ctx)?;

        Ok(LocalComatch {
            span: *span,
            ctx: Some(ctx.vars.clone()),
            name: name.clone(),
            is_lambda_sugar: *is_lambda_sugar,
            cases,
            inferred_type: Some(typ_app),
        })
    }

    fn infer(&self, _prg: &Module, _ctx: &mut Ctx) -> Result<Self, TypeError> {
        Err(TypeError::CannotInferComatch { span: self.span().to_miette() })
    }
}

pub struct WithDestructee<'a> {
    pub cases: &'a Vec<Case>,
    /// Name of the global codefinition that gets substituted for the destructor's self parameters
    pub label: Option<Ident>,
    pub n_label_args: usize,
    pub destructee: TypCtor,
}

/// Infer a copattern match
impl<'a> WithDestructee<'a> {
    /// Check whether the copattern match contains exactly one clause for every
    /// destructor declared in the codata type declaration.
    pub fn check_exhaustiveness(&self, prg: &Module) -> Result<(), TypeError> {
        let WithDestructee { cases, .. } = &self;
        // Check that this comatch is on a codata type
        let codata = prg.codata(&self.destructee.name, self.destructee.span())?;

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

        if (dtors_missing.peek().is_some())
            || dtors_exessive.peek().is_some()
            || !dtors_duplicate.is_empty()
        {
            return Err(TypeError::invalid_match(
                dtors_missing.cloned().collect(),
                dtors_exessive.cloned().collect(),
                dtors_duplicate,
                &self.destructee.span(),
            ));
        }
        Ok(())
    }

    pub fn infer_wd(&self, prg: &Module, ctx: &mut Ctx) -> Result<Vec<Case>, TypeError> {
        let WithDestructee { cases, destructee, n_label_args, label } = &self;
        let TypCtor { args: on_args, .. } = destructee;

        let on_args = on_args.shift((1, 0)); // FIXME: where to shift this

        let mut cases_out = Vec::new();

        for case in cases.iter().cloned() {
            let Dtor { self_param, ret_typ, params, .. } = prg.dtor(&case.name, case.span)?;
            let SelfParam { typ: TypCtor { args: def_args, .. }, .. } = self_param;
            let Case { span, name, params: params_inst, body } = &case;
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
            let def_args =
                def_args.normalize(prg, &mut LevelCtx::from(vec![params.len()]).env())?;
            let ret_typ =
                ret_typ.normalize(prg, &mut LevelCtx::from(vec![params.len(), 1]).env())?;

            // We have to check whether we have an absurd case or an ordinary case.
            // To do this we have solve the following unification problem:
            //
            //               T(...) =? T(...)
            //                 ^^^       ^^^
            //                  |         \----------------------- on_args
            //                  \--------------------------------- def_args
            //
            let eqns: Vec<_> = def_args
                .iter()
                .cloned()
                .zip(on_args.args.iter().cloned())
                .map(|(lhs, rhs)| Eqn { lhs, rhs })
                .collect();

            let ret_typ_nf = match label {
                // Substitute the codef label for the self parameter
                Some(label) => {
                    let args = (0..*n_label_args)
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
                    ret_typ.subst(&mut subst_ctx, &subst).shift((-1, 0)).normalize(
                        prg,
                        &mut LevelCtx::from(vec![*n_label_args, params.len()]).env(),
                    )?
                }
                // TODO: Self parameter for local comatches
                None => ret_typ.shift((-1, 0)),
            };

            // Check the case given the equations
            let case_out = params_inst.check_telescope(
                prg,
                name,
                ctx,
                params,
                |ctx, args_out| {
                    let body_out = match body {
                        Some(body) => {
                            let unif =
                                unify(ctx.levels(), &mut ctx.meta_vars, eqns.clone(), false)?
                                    .map_no(|()| TypeError::PatternIsAbsurd {
                                        name: name.clone(),
                                        span: span.to_miette(),
                                    })
                                    .ok_yes()?;

                            ctx.fork::<Result<_, TypeError>, _>(|ctx| {
                                ctx.subst(prg, &unif)?;
                                let body = body.subst(&mut ctx.levels(), &unif);

                                let t_subst = ret_typ_nf.subst(&mut ctx.levels(), &unif);
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
            )?;

            cases_out.push(case_out);
        }

        Ok(cases_out)
    }
}
