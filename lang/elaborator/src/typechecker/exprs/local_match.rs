//! Bidirectional type checker

use std::cmp;
use std::convert::Infallible;

use ast::ctx::values::{Binder, TypeCtx};
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

/// Compute the annotated motive and type of the body of the clauses if the user has
/// written a local pattern match with a motive:
///
/// ```text
///  e.match as x => t { ... }
///  ^       ^^^^^^^^^
///  |         motive
/// on_exp
/// ```
///
/// # Parameters
///
/// - `ctx`: the typechecking context of the local match.
/// - `motive`: The motive written by the programmer.
/// - `on_exp`: The term on which we pattern match.
/// - `on_exp_typ`: The type of `on_exp`.
/// - `expected_type`: We are in checking mode, so this is the expected type of the entire pattern matching expression.
///
/// # Requires
///
/// - `on_exp` is already fully inferred.
/// - `on_exp_typ` is the type of `on_exp`.
///
/// # Returns
/// - The fully inferred and annotated motive
/// - The type that we should use to check the RHSs of the individual clauses.
///   (Before the refinements introduces through dependent pattern matching.)
///
fn compute_motive(
    ctx: &mut Ctx,
    motive: &Motive,
    on_exp: Box<Exp>,
    on_exp_typ: TypCtor,
    expected_type: &Exp,
) -> TcResult<(Motive, Box<Exp>)> {
    let Motive { span, param, ret_typ } = motive;
    let mut self_t_nf = on_exp_typ.to_exp().normalize(&ctx.type_info_table, &mut ctx.env())?;
    self_t_nf.shift((1, 0));
    let self_binder = Binder { name: param.name.clone(), content: self_t_nf.clone() };

    // Typecheck the motive
    let ret_typ_out = ctx.bind_single(self_binder.clone(), |ctx| {
        ret_typ.check(ctx, &Box::new(TypeUniv::new().into()))
    })?;

    // Ensure that the motive matches the expected type
    let motive_binder = Binder { name: motive.param.name.clone(), content: () };
    let mut subst_ctx = ctx.levels().append(&vec![vec![motive_binder]].into());
    let on_exp = shift_and_clone(&on_exp, (1, 0));
    let subst = Assign { lvl: Lvl { fst: subst_ctx.len() - 1, snd: 0 }, exp: on_exp };
    let mut motive_t = ret_typ.subst(&mut subst_ctx, &subst)?;
    motive_t.shift((-1, 0));
    let motive_t_nf = motive_t.normalize(&ctx.type_info_table, &mut ctx.env())?;
    convert(&ctx.vars, &mut ctx.meta_vars, motive_t_nf, expected_type, span)?;

    let body_t = ctx.bind_single(self_binder.clone(), |ctx| {
        ret_typ.normalize(&ctx.type_info_table, &mut ctx.env())
    })?;
    let motive_out = Motive {
        span: *span,
        param: ParamInst {
            span: *span,
            name: param.name.clone(),
            typ: Box::new(on_exp_typ.to_exp()).into(),
            erased: param.erased,
        },
        ret_typ: ret_typ_out,
    };
    Ok((motive_out, body_t))
}

impl CheckInfer for LocalMatch {
    fn check(&self, ctx: &mut Ctx, t: &Exp) -> TcResult<Self> {
        let LocalMatch { span, name, on_exp, motive, cases, .. } = self;

        // Compute the type of the expression we are pattern matching on.
        // This should always be a type constructor for a data type.
        let on_exp_out = on_exp.infer(ctx)?;
        let on_exp_typ = on_exp_out.expect_typ()?.expect_typ_app()?;

        // Compute the new motive and the type that we should use to check the bodies of the cases.
        let (motive_out, _) = match motive {
            // Pattern matching with motive
            Some(motive) => {
                let (motive, body_typ) =
                    compute_motive(ctx, motive, on_exp.clone(), on_exp_typ.clone(), t)?;
                (Some(motive), body_typ)
            }
            // Pattern matching without motive
            None => (None, Box::new(shift_and_clone(t, (1, 0)))),
        };

        let Cases::Unchecked { cases } = cases else { unreachable!() };

        let mut fvs = motive.free_vars_closure(&ctx.vars);
        fvs.extend(t.free_vars_closure(&ctx.vars));
        fvs.extend(cases.free_vars_closure(&ctx.vars));

        let def_name = unique_def_name(name, &on_exp_typ.name.id);

        let LiftedSignature { telescope: params, subst, args } = lifted_signature(&ctx.vars, fvs);

        // Substitute the new parameters for the free variables
        // Unwrap is safe here because we are unwrapping an infallible result
        let def_cases = cases.subst(&mut ctx.levels(), &subst).unwrap();
        let def_self_typ = on_exp_typ.subst(&mut ctx.levels(), &subst).unwrap();
        let def_ret_typ = match &motive {
            Some(m) => m.subst(&mut ctx.levels(), &subst).unwrap().ret_typ,
            None => shift_and_clone(
                &Box::new(t.clone().subst(&mut ctx.levels(), &subst).unwrap()),
                (1, 0),
            ),
        };

        let def = Def {
            // FIXME: How do we ensure good error messages?
            span: *span,
            doc: None,
            name: def_name.clone(),
            attr: Attributes::default(),
            params,
            self_param: SelfParam {
                info: None,
                name: motive
                    .as_ref()
                    .map(|m| m.param.name.clone())
                    .unwrap_or(VarBind::Wildcard { span: None }),
                typ: def_self_typ,
            },
            ret_typ: def_ret_typ,
            cases: def_cases,
        };

        ctx.lifted_decls.push(def.into());

        let lifted_def = IdBound { span: None, id: def_name.id, uri: ctx.module.uri.clone() };

        Ok(LocalMatch {
            span: *span,
            ctx: Some(ctx.vars.clone()),
            name: name.clone(),
            on_exp: on_exp_out,
            motive: motive_out,
            ret_typ: Some(Box::new(t.clone())),
            cases: Cases::Checked { cases: cases.clone(), args, lifted_def },
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

/// Sort the variables such the dependency relation is satisfied
/// Due to unification, it is not sufficient to sort them according to their De-Bruijn level:
/// Unification can lead to a set of variables where variables with a higher De-Bruijn level
/// may occur in the types of variables with a lower De-Bruijn level.
/// This is because unification may locally refine types.
/// Example:
///
/// ```pol
/// data Bar(a: Type) { }
///
/// codata Baz { unit: Top }
///
/// data Foo(a: Type) {
///    MkFoo(a: Type): Foo(Bar(a)),
/// }
///
/// data Top { Unit }
///
/// def Top.ignore(a: Type, x: a): Top {
///     Unit => Unit
/// }
///
/// def Top.foo(a: Type, foo: Foo(a)): Baz {
///     Unit => foo.match {
///         MkFoo(a') => comatch {
///            unit => Unit.ignore(Foo(Bar(a')), foo)
///        }
///    }
/// }
/// ```
///
/// In this example, unification may perform the substitution `{a := a'}` such that locally
/// the type of foo is known to be `Foo(Bar(a'))`.
/// Hence, lifting of the comatch will need to consider the variables [ foo: Foo(Bar(a')), a': Type ]
/// where `foo` depends on `a'` even though it has been bound earlier in the context
fn sort_vars(ctx: &TypeCtx, fvs: HashSet<Lvl>) -> Vec<Lvl> {
    let mut fvs: Vec<_> = fvs.into_iter().collect();
    fvs.sort_by(|x, y| cmp_vars(ctx, *x, *y));
    fvs
}

fn cmp_vars(ctx: &TypeCtx, x: Lvl, y: Lvl) -> cmp::Ordering {
    let x_typ = ctx.lookup(x);
    let y_typ = ctx.lookup(y);
    let x_occurs_in_y = y_typ.occurs_var(&mut ctx.levels(), x);
    let y_occurs_in_x = x_typ.occurs_var(&mut ctx.levels(), y);
    assert!(!(x_occurs_in_y && y_occurs_in_x));
    if x_occurs_in_y {
        cmp::Ordering::Less
    } else if y_occurs_in_x {
        cmp::Ordering::Greater
    } else {
        x.cmp(&y)
    }
}

/// Generate a definition name based on the label and type information
fn unique_def_name(label: &Label, type_name: &str) -> IdBind {
    label.user_name.clone().unwrap_or_else(|| {
        let lowered = type_name.to_lowercase();
        let id = label.id;
        IdBind::from_string(&format!("d_{lowered}{id}"))
    })
}

/// Compute the lifted signature based on the set of free variables of an expression `e`.
/// Using the lifted signature `LiftedSignature { telescope, subst, args }`, the lifted expression
///
/// ```text
/// let f(telescope) { e[subst] }
/// ```
///
/// can be constructed, where `f` is a fresh name. The expression `e` can be replaced by `f(args)`.
///
/// # Parameters
///
/// - `fvs`: Set of free variables
/// - `base_ctx`: Context under which the expression `e` is well-typed
///
/// # Requires
///
/// - `fvs ⊆ base_ctx`
///
/// # Returns
///
/// The signature `LiftedSignature { telescope, subst, args }` of the lifted expression, consisting of:
///
/// - `telescope`: The telescope under which `e[subst]` is closed
/// - `subst`: A substitution that closes the free variables under `telescope`
/// - `args`: The arguments to apply to the lifted expression
///
/// # Ensures
///
/// - `telescope ⊆ base_ctx`
/// - `let f(telescope) { e[subst] }` is well-typed in the empty context
/// - `e = f(args)` and well-typed in `base_ctx`
///
pub fn lifted_signature(ctx: &TypeCtx, fvs: HashSet<Lvl>) -> LiftedSignature {
    let cutoff = ctx.len();
    // Sort the list of free variables by the De-Bruijn level such the dependency relation is satisfied.
    // Types can only depend on types which occur earlier in the context.
    let fvs_sorted = sort_vars(ctx, fvs);

    let mut params: Vec<Param> = vec![];
    let mut args = vec![];

    let mut subst = CloseParamsSubst { subst: HashMap::default(), cutoff };

    for lvl in fvs_sorted.into_iter() {
        let binder = ctx.lookup(lvl);
        let name = match &binder.name {
            ast::VarBind::Var { id, .. } => VarBound::from_string(id),
            // When we encouter a wildcard, we use `x` as a placeholder name for the variable referencing this binder.
            // Of course, `x` is not guaranteed to be unique; in general we do not guarantee that the string representation of variables remains intact during elaboration.
            // When reliable variable names are needed (e.g. for printing source code or code generation), the `renaming` transformation needs to be applied to the AST first.
            ast::VarBind::Wildcard { .. } => VarBound::from_string("x"),
        };

        // Unwrap is safe here because we are unwrapping an infallible result
        let typ = binder.content.subst(&mut ctx.levels(), &subst).unwrap();

        let param = Param {
            implicit: false,
            name: VarBind::from_string(&name.id),
            typ: typ.clone(),
            erased: false,
        };
        let arg = Arg::UnnamedArg {
            arg: Box::new(Exp::Variable(Variable {
                span: None,
                idx: ctx.lvl_to_idx(lvl),
                name: name.clone(),
                inferred_type: None,
            })),
            erased: false,
        };
        args.push(arg);
        params.push(param);
        subst.add(name, lvl);
    }

    LiftedSignature {
        telescope: Telescope { params },
        subst: subst.into_body_subst(),
        args: Args { args },
    }
}

/// The signature of a lifted expression
pub struct LiftedSignature {
    /// Telescope of the lifted expression
    pub telescope: Telescope,
    /// Substitution that is applied to the body of the lifted expression
    pub subst: CloseBodySubst,
    /// An instantiation of `telescope` with the free variables
    pub args: Args,
}

/// Substitution applied to parameters of the new top-level definition
#[derive(Clone, Debug)]
pub struct CloseParamsSubst {
    /// Mapping of the original De-Bruijn levels of a free variable to the new binders
    subst: HashMap<Lvl, (Lvl, VarBound)>,
    /// The De-Bruijn level (fst index) up to which a variable counts as free
    cutoff: usize,
}

impl CloseParamsSubst {
    fn add(&mut self, name: VarBound, lvl: Lvl) {
        self.subst.insert(lvl, (Lvl { fst: 0, snd: self.subst.len() }, name));
    }

    /// Build the substitution applied to the body of the new definition
    fn into_body_subst(self) -> CloseBodySubst {
        CloseBodySubst { subst: self.subst, cutoff: self.cutoff }
    }
}

impl Shift for CloseParamsSubst {
    fn shift_in_range<R: ShiftRange>(&mut self, _range: &R, _by: (isize, isize)) {
        // Since FVSubst works with levels, it is shift-invariant
    }
}

impl Substitution for CloseParamsSubst {
    type Err = Infallible;

    fn get_subst(&self, _ctx: &LevelCtx, lvl: Lvl) -> Result<Option<Box<Exp>>, Self::Err> {
        Ok(self.subst.get(&lvl).map(|(lvl, name)| {
            Box::new(Exp::Variable(Variable {
                span: None,
                idx: Idx { fst: 0, snd: self.subst.len() - 1 - lvl.snd },
                name: name.clone(),
                inferred_type: None,
            }))
        }))
    }
}

/// Substitution applied to the body of the new definition
#[derive(Clone, Debug)]
pub struct CloseBodySubst {
    /// Mapping of the original De-Bruijn levels of a free variable to the new reference
    subst: HashMap<Lvl, (Lvl, VarBound)>,
    /// The De-Bruijn level (fst index) up to which a variable counts as free
    cutoff: usize,
}

impl Shift for CloseBodySubst {
    fn shift_in_range<R: ShiftRange>(&mut self, _range: &R, _by: (isize, isize)) {
        // Since FVSubst works with levels, it is shift-invariant
    }
}

impl Substitution for CloseBodySubst {
    type Err = Infallible;

    fn get_subst(&self, ctx: &LevelCtx, lvl: Lvl) -> Result<Option<Box<Exp>>, Self::Err> {
        // Let Γ be the original context, let Δ be the context according to which the new De-Bruijn index should be calculated
        //
        // Γ = [[x], [y], [z]]
        //     ^^^^^^^^  ^
        //    free vars  cutoff
        //
        // Δ = [[x, y], [z]]
        //      ^^^^^^  ^^^ bound vars
        // new telescope

        // Compute the names for the free variables in the correct order
        // This is only needed to satisfy LevelCtx now tracking the names of the binders.
        // FIXME: This needs to be refactored
        let mut free_vars = self.subst.iter().collect::<Vec<_>>();
        free_vars.sort_by_key(|(lvl, _)| *lvl);
        let free_vars = free_vars
            .into_iter()
            .map(|(_, (_, name))| VarBind::from_string(&name.id))
            .collect::<Vec<_>>();
        let new_ctx = LevelCtx::from(vec![free_vars]).append(&ctx.tail(self.cutoff));
        Ok(self.subst.get(&lvl).map(|(lvl, name)| {
            Box::new(Exp::Variable(Variable {
                span: None,
                idx: new_ctx.lvl_to_idx(*lvl),
                name: name.clone(),
                inferred_type: None,
            }))
        }))
    }
}
