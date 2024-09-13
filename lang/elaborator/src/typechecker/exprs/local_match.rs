//! Bidirectional type checker

use std::rc::Rc;

use crate::normalizer::env::ToEnv;
use crate::normalizer::normalize::Normalize;
use crate::typechecker::exprs::CheckTelescope;
use crate::typechecker::lookup_table::CtorMeta;
use crate::unifier::constraints::Constraint;
use crate::unifier::unify::*;
use miette_util::ToMiette;
use syntax::ast::*;
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
    fn check(&self, ctx: &mut Ctx, t: Rc<Exp>) -> Result<Self, TypeError> {
        let LocalMatch { span, name, on_exp, motive, cases, .. } = self;
        let on_exp_out = on_exp.infer(ctx)?;
        let typ_app_nf = on_exp_out
            .typ()
            .ok_or(TypeError::Impossible {
                message: "Expected inferred type".to_owned(),
                span: None,
            })?
            .expect_typ_app()?;
        let typ_app = typ_app_nf.infer(ctx)?;
        let ret_typ_out = t.check(ctx, Rc::new(TypeUniv::new().into()))?;

        let motive_out;
        let body_t;

        match motive {
            // Pattern matching with motive
            Some(m) => {
                let Motive { span: info, param, ret_typ } = m;
                let self_t_nf =
                    typ_app.to_exp().normalize(&ctx.module, &mut ctx.env())?.shift((1, 0));
                let self_binder = Binder { name: param.name.clone(), typ: self_t_nf.clone() };

                // Typecheck the motive
                let ret_typ_out = ctx.bind_single(&self_binder, |ctx| {
                    ret_typ.check(ctx, Rc::new(TypeUniv::new().into()))
                })?;

                // Ensure that the motive matches the expected type
                let mut subst_ctx = ctx.levels().append(&vec![1].into());
                let on_exp_shifted = on_exp.shift((1, 0));
                let subst =
                    Assign { lvl: Lvl { fst: subst_ctx.len() - 1, snd: 0 }, exp: on_exp_shifted };
                let motive_t = ret_typ.subst(&mut subst_ctx, &subst).shift((-1, 0));
                let motive_t_nf = motive_t.normalize(&ctx.module, &mut ctx.env())?;
                convert(subst_ctx, &mut ctx.meta_vars, motive_t_nf, &t)?;

                body_t = ctx.bind_single(&self_binder, |ctx| {
                    ret_typ.normalize(&ctx.module, &mut ctx.env())
                })?;
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

        let ws = WithScrutinee { cases, scrutinee: typ_app_nf.clone() };
        ws.check_exhaustiveness(&ctx.module)?;
        let cases = ws.check_ws(ctx, body_t)?;

        Ok(LocalMatch {
            span: *span,
            ctx: Some(ctx.vars.clone()),
            name: name.clone(),
            on_exp: on_exp_out,
            motive: motive_out,
            ret_typ: ret_typ_out.into(),
            cases,
            inferred_type: Some(typ_app),
        })
    }

    fn infer(&self, __ctx: &mut Ctx) -> Result<Self, TypeError> {
        Err(TypeError::CannotInferMatch { span: self.span().to_miette() })
    }
}

pub struct WithScrutinee<'a> {
    pub cases: &'a Vec<Case>,
    pub scrutinee: TypCtor,
}

/// Check a pattern match
impl<'a> WithScrutinee<'a> {
    /// Check whether the pattern match contains exactly one clause for every
    /// constructor declared in the data type declaration.
    pub fn check_exhaustiveness(&self, module: &Module) -> Result<(), TypeError> {
        let WithScrutinee { cases, .. } = &self;
        // Check that this match is on a data type
        let data =
            module.lookup_data(&self.scrutinee.name).ok_or_else(|| TypeError::Impossible {
                message: format!("Data type {} not found", self.scrutinee.name),
                span: None,
            })?;

        // Check exhaustiveness
        let ctors_expected: HashSet<_> =
            data.ctors.iter().map(|ctor| ctor.name.to_owned()).collect();
        let mut ctors_actual = HashSet::default();
        let mut ctors_duplicate = HashSet::default();

        for name in cases.iter().map(|case| &case.pattern.name) {
            if ctors_actual.contains(name) {
                ctors_duplicate.insert(name.clone());
            }
            ctors_actual.insert(name.clone());
        }
        let mut ctors_missing = ctors_expected.difference(&ctors_actual).peekable();
        let mut ctors_undeclared = ctors_actual.difference(&ctors_expected).peekable();

        if (ctors_missing.peek().is_some())
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
        Ok(())
    }

    pub fn check_ws(&self, ctx: &mut Ctx, t: Rc<Exp>) -> Result<Vec<Case>, TypeError> {
        let WithScrutinee { cases, .. } = &self;

        let cases: Vec<_> = cases.to_vec();
        let mut cases_out = Vec::new();

        for case in cases {
            let Case { span, pattern: Pattern { name, params: args, .. }, body } = case;
            // Build equations for this case
            let CtorMeta { typ: TypCtor { args: def_args, .. }, params, .. } =
                ctx.lookup_table.lookup_ctor(&name)?;

            let def_args_nf = LevelCtx::empty().bind_iter(params.params.iter(), |ctx_| {
                def_args.normalize(&ctx.module, &mut ctx_.env())
            })?;

            let TypCtor { args: on_args, .. } = &self.scrutinee;
            let on_args = on_args.shift((1, 0)); // FIXME: where to shift this

            // Check the case given the equations
            // FIXME: Refactor this
            let mut subst_ctx_1 = ctx.levels().append(&vec![1, params.len()].into());
            let mut subst_ctx_2 = ctx.levels().append(&vec![params.len(), 1].into());
            let curr_lvl = subst_ctx_2.len() - 1;

            let module = ctx.module.clone();
            let name = name.clone();
            let params = params.clone();

            args.check_telescope(
                &name,
                ctx,
                &params,
                |ctx, args_out| {
                    // Substitute the constructor for the self parameter
                    let args = (0..params.len())
                        .rev()
                        .map(|snd| {
                            Arg::UnnamedArg(Rc::new(Exp::Variable(Variable {
                                span: None,
                                idx: Idx { fst: 1, snd },
                                name: "".to_owned(),
                                inferred_type: None,
                            })))
                        })
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

                    let constraint =
                        Constraint::EqualityArgs { lhs: Args { args: def_args_nf }, rhs: on_args };

                    let body_out = match body {
                        Some(body) => {
                            let unif = unify(ctx.levels(), &mut ctx.meta_vars, constraint, false)?
                                .map_no(|()| TypeError::PatternIsAbsurd {
                                    name: name.clone(),
                                    span: span.to_miette(),
                                })
                                .ok_yes()?;

                            ctx.fork::<Result<_, TypeError>, _>(|ctx| {
                                ctx.subst(&module, &unif)?;
                                let body = body.subst(&mut ctx.levels(), &unif);

                                let t_subst = t.subst(&mut ctx.levels(), &unif);
                                let t_nf = t_subst.normalize(&module, &mut ctx.env())?;

                                let body_out = body.check(ctx, t_nf)?;

                                Ok(Some(body_out))
                            })?
                        }
                        None => {
                            unify(ctx.levels(), &mut ctx.meta_vars, constraint, false)?
                                .map_yes(|_| TypeError::PatternIsNotAbsurd {
                                    name: name.clone(),
                                    span: span.to_miette(),
                                })
                                .ok_no()?;

                            None
                        }
                    };
                    let case_out = Case {
                        span,
                        pattern: Pattern {
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
