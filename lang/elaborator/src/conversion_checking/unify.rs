//! This file implements the core logic for unification for conversion checking
//!
//! It is based on the following references:
//!
//! * Andreas Abel, and Brigitte Pientka. "Higher-order dynamic pattern unification for dependent types and records." (2011)
//! * Adam Gundry and Conor McBride. "A tutorial implementation of dynamic pattern unification." (2013).
//! * András Kovács's elaboration-zoo (https://github.com/AndrasKovacs/elaboration-zoo)

use ast::{ctx::values::Binder, Variable};
use ctx::LevelCtx;
use miette_util::{codespan::Span, ToMiette};

use ast::*;
use printer::Print;

use crate::result::{TcResult, TypeError};

use super::constraints::Constraint;

pub struct Ctx<'a> {
    /// Constraints that have not yet been solved
    pub constraints: Vec<Constraint<'a>>,
    /// A cache of solved constraints. We can skip solving a constraint
    /// if we have seen it before
    pub done: HashSet<Constraint<'a>>,
}

impl<'a> Ctx<'a> {
    pub fn new(constraints: Vec<Constraint<'a>>) -> Self {
        Self { constraints, done: HashSet::default() }
    }

    pub fn unify(
        &mut self,
        meta_vars: &mut HashMap<MetaVar, MetaVarState>,
        while_elaborating_span: &Option<Span>,
    ) -> TcResult {
        while let Some(constraint) = self.constraints.pop() {
            self.unify_eqn(&constraint, meta_vars, while_elaborating_span)?;
            self.done.insert(constraint);
        }
        Ok(())
    }

    fn unify_eqn(
        &mut self,
        eqn: &Constraint<'a>,
        meta_vars: &mut HashMap<MetaVar, MetaVarState>,
        while_elaborating_span: &Option<Span>,
    ) -> TcResult {
        match eqn {
            Constraint::Equality { ctx: constraint_cxt, lhs, rhs } => match (&**lhs, &**rhs) {
                // This is the most interesting case, where we equate a hole with an expression.
                (Exp::Hole(h), candidate) | (candidate, Exp::Hole(h)) => {
                    // First, we check if the hole has already been solved.
                    // In that case, we only need to substitute the arguments in the solution.
                    // For instance, this is the case for any holes in declarations imported from other modules.
                    // For these, it is particularly important to have this early-return, because metavariables
                    // from other modules are not bound in the metavars map for this module!
                    if let Some(solution) = &h.solution {
                        let lhs = solution.clone().subst(&mut h.levels(), &h.args)?;
                        self.add_constraint(Constraint::Equality {
                            ctx: constraint_cxt,
                            lhs,
                            rhs: Box::new(candidate.clone()),
                        })?;
                        return Ok(());
                    }
                    // Every hole is associated with a metavariable and a list of arguments.
                    // The unification problem is:
                    //
                    // ```text
                    // constraint_cxt ⊢ h.metavar args =? candidate
                    // ```
                    let metavar_state = meta_vars.get(&h.metavar).unwrap();
                    match metavar_state {
                        // When we encounter an unsolved metavariable, we attempt to solve it
                        MetaVarState::Unsolved { ctx: metavar_ctx } => self.solve_meta_var(
                            meta_vars,
                            metavar_ctx.clone(),
                            h.metavar,
                            &h.args,
                            constraint_cxt.levels(),
                            Box::new(candidate.clone()),
                            while_elaborating_span,
                        )?,
                        // When we encounter a solved metavariable, we substitute the arguments in the solution.
                        MetaVarState::Solved { ctx, solution } => {
                            let lhs = solution.clone().subst(&mut ctx.clone(), &h.args)?;
                            self.add_constraint(Constraint::Equality {
                                ctx: constraint_cxt,
                                lhs,
                                rhs: Box::new(candidate.clone()),
                            })?;
                        }
                    }

                    Ok(())
                }
                (
                    Exp::Variable(v1 @ Variable { idx: idx_1, .. }),
                    Exp::Variable(v2 @ Variable { idx: idx_2, .. }),
                ) => {
                    if idx_1 == idx_2 {
                        Ok(())
                    } else {
                        Err(TypeError::NotEqInternal {
                            lhs: v1.print_to_string(None),
                            rhs: v2.print_to_string(None),
                        }
                        .into())
                    }
                }
                (Exp::Variable(v @ Variable { .. }), other) => Err(TypeError::NotEqInternal {
                    lhs: v.print_to_string(None),
                    rhs: other.print_to_string(None),
                }
                .into()),
                (other, Exp::Variable(v @ Variable { .. })) => Err(TypeError::NotEqInternal {
                    lhs: other.print_to_string(None),
                    rhs: v.print_to_string(None),
                }
                .into()),
                (
                    Exp::TypCtor(TypCtor { name, args, .. }),
                    Exp::TypCtor(TypCtor { name: name2, args: args2, .. }),
                ) if name == name2 => {
                    let constraint = Constraint::EqualityArgs {
                        ctx: constraint_cxt,
                        lhs: args.clone(),
                        rhs: args2.clone(),
                    };
                    self.add_constraint(constraint)
                }
                (
                    Exp::TypCtor(t1 @ TypCtor { name, .. }),
                    Exp::TypCtor(t2 @ TypCtor { name: name2, .. }),
                ) if name != name2 => Err(TypeError::NotEqInternal {
                    lhs: t1.print_to_string(None),
                    rhs: t2.print_to_string(None),
                }
                .into()),
                (
                    Exp::Call(Call { name, args, .. }),
                    Exp::Call(Call { name: name2, args: args2, .. }),
                ) if name == name2 => {
                    let constraint = Constraint::EqualityArgs {
                        ctx: constraint_cxt,
                        lhs: args.clone(),
                        rhs: args2.clone(),
                    };
                    self.add_constraint(constraint)
                }
                (Exp::Call(c1 @ Call { name, .. }), Exp::Call(c2 @ Call { name: name2, .. }))
                    if name != name2 =>
                {
                    Err(TypeError::NotEqInternal {
                        lhs: c1.print_to_string(None),
                        rhs: c2.print_to_string(None),
                    }
                    .into())
                }
                (
                    Exp::DotCall(DotCall { exp, name, args, .. }),
                    Exp::DotCall(DotCall { exp: exp2, name: name2, args: args2, .. }),
                ) if name == name2 => {
                    self.add_constraint(Constraint::Equality {
                        ctx: constraint_cxt,
                        lhs: exp.clone(),
                        rhs: exp2.clone(),
                    })?;
                    let constraint = Constraint::EqualityArgs {
                        ctx: constraint_cxt,
                        lhs: args.clone(),
                        rhs: args2.clone(),
                    };
                    self.add_constraint(constraint)
                }
                (Exp::TypeUniv(_), Exp::TypeUniv(_)) => Ok(()),
                (Exp::Anno(Anno { exp, .. }), rhs) => self.add_constraint(Constraint::Equality {
                    ctx: constraint_cxt,
                    lhs: exp.clone(),
                    rhs: Box::new(rhs.clone()),
                }),
                (lhs, Exp::Anno(Anno { exp, .. })) => self.add_constraint(Constraint::Equality {
                    ctx: constraint_cxt,
                    lhs: Box::new(lhs.clone()),
                    rhs: exp.clone(),
                }),
                (
                    Exp::LocalComatch(LocalComatch { name: name_lhs, cases: cases_lhs, .. }),
                    Exp::LocalComatch(LocalComatch { name: name_rhs, cases: cases_rhs, .. }),
                ) if name_lhs == name_rhs => {
                    let new_eqns =
                        zip_cases_by_xtors(cases_lhs, cases_rhs).filter_map(|(lhs, rhs)| {
                            if let (Some(lhs), Some(rhs)) = (lhs.body, rhs.body) {
                                Some(Constraint::Equality { ctx: constraint_cxt, lhs, rhs })
                            } else {
                                None
                            }
                        });
                    self.add_constraints(new_eqns)
                }
                (_, _) => Err(TypeError::cannot_decide(lhs, rhs, while_elaborating_span)),
            },
            Constraint::EqualityArgs { ctx: constraint_ctx, lhs, rhs } => {
                let new_eqns =
                    lhs.args.iter().cloned().zip(rhs.args.iter().cloned()).map(|(lhs, rhs)| {
                        Constraint::Equality {
                            ctx: constraint_ctx,
                            lhs: lhs.exp().clone(),
                            rhs: rhs.exp().clone(),
                        }
                    });
                self.add_constraints(new_eqns)?;
                Ok(())
            }
        }
    }

    fn add_constraint(&mut self, eqn: Constraint<'a>) -> TcResult {
        self.add_constraints([eqn])
    }

    fn add_constraints<I: IntoIterator<Item = Constraint<'a>>>(&mut self, iter: I) -> TcResult {
        self.constraints.extend(iter.into_iter().filter(|eqn| !self.done.contains(eqn)));
        Ok(())
    }

    /// Attempt to solve a metavariable with a given candidate solution.
    /// In particular, this method attempts to solve the following equation:
    ///
    /// ```text
    /// constraint_ctx ⊢ metavar args =? candidate
    /// ```
    ///
    /// Let us start with a quick note on the contexts involved here.
    /// First of all, at the site where it is introduced, `metavar` is a well-typed hole under a context which
    /// we will call `metavar_ctx`.
    /// However, when this method is called we also know that, by assumption,
    /// `constraint_ctx  ⊢ candidate` as well as `constraint_ctx ⊢ metavar args`.
    /// Our goal is to transform `candidate` in such a way that it can be inserted for `metavar`.
    /// Let's call the transformed `candidate` expression `solution`.
    /// After the transformation we will have `metavar_ctx ⊢ solution`.
    ///
    /// In general, this problem is undecidable and there is not always a unique solution.
    /// Therefore, we only accept a decidable subset of unification problems, namely those that fall
    /// under Millner's pattern fragment:
    ///
    /// 1. `args` consists of distinct bound variables
    /// 2. every free variable of `candidate` occurs in `args`
    /// 3. `metavar` does not occur in `candidate`
    ///
    /// If these conditions are met, there exists the following unique solution:
    ///
    /// ```text
    /// metavar = candidate[args⁻¹]
    /// ```
    ///
    /// where `args⁻¹` is the "inverse" substitution to `args`.
    /// The inverse substitution maps each argument in `args` to the corresponding bound variable in `metavar_ctx`.
    /// For instance, if `constraint_ctx = [[x, y], [z]]`, `metavar_ctx = [[a, b]]`, and `args = (x, y)`,
    /// then `args⁻¹ = { x ↦ a, y ↦ b }`. If `candidate = SomeCtor(x)`, then `meta_var` is solved to `SomeCtor(a)`.
    ///
    /// # Examples
    ///
    /// First, consider the following simple example:
    ///
    /// ```text
    /// 1: let n1 : Nat = 5;
    /// 2: let n2 : Nat = _;
    /// 3: let n3 : Nat = 2;
    /// 4: Refl(n2) : n1 = n1
    /// ```
    ///
    /// Here, we have that:
    ///
    /// - `meta_var_cxt = [n1 : Nat]` (at 2:)
    /// - `constraint_ctx = [n1: Nat, n2: Nat, n3: Nat]` (at 4:)
    ///
    /// We have to solve `n2 =? n1`. The arguments of the metavariable (at 2:) are `args = (n1)`.
    /// The candidate solution is `candidate = n1`, `args⁻¹ = { n1 ↦ n1 }` and hence we solve the problem with
    /// `solution = n1[args⁻¹] = n1`.
    ///
    /// Now let us look at a little less straightforward example.
    ///
    /// ```text
    /// let bar(a: Type, x: ?0 (a) ): a { ... }
    /// let force(b: Type, x: b): b { bar(b, x) }
    /// ```
    ///
    /// Here, the type of `x` in `bar` is left out and a metavariable `?0` generated for the hole.
    /// Solving this metavariable is forced in the call to `bar` in `force`.
    /// In particular, we check that `(b, x): [a: Type, x: ?0 (a)]`, which equates `?0 =? b`.
    /// Hence, we have that:
    ///
    /// - `metavar_ctx = [a: Type] ⊢ ?0(a) : Type`
    /// - `constraint_ctx = [b: Type, x: b] ⊢ ?0 = b : Type`
    /// - `args = (b)`
    /// - `args⁻¹ = { b ↦ a }``
    /// - `solution = b[args⁻¹] = a`
    ///
    /// # Parameters
    ///
    /// - `meta_vars`: The global metavariables state
    /// - `metavar_ctx`: The context of the metavariable
    /// - `metavar`: The metavariable to solve
    /// - `args`: The arguments of the metavariable
    /// - `constraint_ctx`: The context of the solution
    /// - `candidate`: The candidate solution
    /// - `while_elaborating_span`: The origin of the unification call,
    ///   tracked here for better error messages
    ///
    /// # Requires
    ///
    /// - `metavar_ctx ⊢ metavar`
    /// - `constraint_ctx ⊢ metavar args`
    /// - `constraint_ctx ⊢ candidate`
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the metavariable was successfully solved
    /// - `Err(TypeError)` if the metavariable could not be solved
    /// - `meta_vars[meta_var]`
    ///
    /// # Ensures
    ///
    /// If solving is successful:
    ///
    /// - `meta_vars[meta_var] = Solved { ctx: metavar_ctx, solution }`
    ///   where `solution` is the solution to the metavariable
    /// - `metavar_ctx ⊢ solution[args]` is well-typed
    /// - `solution` does not contain any other solved metavariables
    /// - All other `meta_vars` do not contain `meta_var` (i.e. `meta_var` is replaced by `solution` via zonking)
    #[allow(clippy::vec_box)]
    #[allow(clippy::too_many_arguments)]
    fn solve_meta_var(
        &mut self,
        meta_vars: &mut HashMap<MetaVar, MetaVarState>,
        metavar_ctx: LevelCtx,
        metavar: MetaVar,
        args: &[Vec<Binder<Box<Exp>>>],
        constraint_ctx: LevelCtx,
        candidate: Box<Exp>,
        while_elaborating_span: &Option<Span>,
    ) -> TcResult {
        log::trace!("Attempting to solve metavariable {}", metavar.print_trace());
        log::trace!("metavar_ctx = {}", metavar_ctx.print_trace());
        log::trace!("metavar = {}", metavar.print_trace());
        log::trace!("args = {}", args.to_vec().print_trace());
        log::trace!("constraint_ctx = {}", constraint_ctx.print_trace());
        log::trace!("candidate = {}", candidate.print_trace());

        // Condition 3: `metavar` does not occur in `candidate`
        if candidate.occurs_metavar(&mut constraint_ctx.clone(), &metavar) {
            return Err(TypeError::MetaOccursCheckFailed {
                span: metavar.span.to_miette(),
                meta_var: metavar.print_to_string(None),
                while_elaborating_span: while_elaborating_span.to_miette(),
            }
            .into());
        }

        let subst = PartialRenaming::from_args(
            &constraint_ctx,
            &metavar_ctx,
            &metavar,
            args,
            while_elaborating_span,
        )?;
        let mut solution = candidate.subst(&mut constraint_ctx.clone(), &subst)?;
        solution
            .zonk(meta_vars)
            .map_err(|err| TypeError::Impossible { message: err.to_string(), span: None })?;
        log::trace!("solution = {}", solution.print_trace());
        meta_vars.insert(metavar, MetaVarState::Solved { ctx: metavar_ctx.clone(), solution });
        let meta_vars_snapshot = meta_vars.clone();
        for (_, state) in meta_vars.iter_mut() {
            match state {
                MetaVarState::Solved { ctx: _, solution } => {
                    solution.zonk(&meta_vars_snapshot).map_err(|err| TypeError::Impossible {
                        message: err.to_string(),
                        span: None,
                    })?;
                }
                MetaVarState::Unsolved { ctx: _ } => {
                    // Nothing to do for unsolved metavariables
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct PartialRenaming {
    /// Map from the metavariable arguments to the levels in the solution
    args_map: HashMap<Lvl, HashSet<Lvl>>,
    /// The names of the variables supplied as arguments to the metavariable, tracked here for better error messages
    arg_var_names: HashMap<Lvl, String>,
    /// The context of the metavariable (and its solution)
    metavar_ctx: LevelCtx,
    /// The metavariable, tracked here for better error messages
    meta_var: MetaVar,
    /// The origin of the unification call, tracked here for better error messages
    while_elaborating_span: Option<Span>,
}

impl PartialRenaming {
    #[allow(clippy::vec_box)]
    fn from_args(
        constraint_ctx: &LevelCtx,
        metavar_ctx: &LevelCtx,
        meta_var: &MetaVar,
        args: &[Vec<Binder<Box<Exp>>>],
        while_elaborating_span: &Option<Span>,
    ) -> TcResult<Self> {
        let mut args_map = HashMap::default();
        let mut arg_var_names = HashMap::default();

        for (fst, telescope) in args.iter().enumerate() {
            for (snd, arg) in telescope.iter().enumerate() {
                let to_lvl = Lvl { fst, snd };
                let Some(Variable { idx, name, .. }) = expect_variable(&arg.content) else {
                    // Condition 1: `args` consists of distinct bound *variables*
                    return Err(TypeError::MetaArgNotVariable {
                        span: meta_var.span.to_miette(),
                        meta_var: meta_var.print_to_string(None),
                        arg: arg.print_to_string(None),
                        while_elaborating_span: while_elaborating_span.to_miette(),
                    }
                    .into());
                };
                let from_lvl = constraint_ctx.idx_to_lvl(*idx);
                args_map.entry(from_lvl).or_insert(HashSet::default()).insert(to_lvl);
                arg_var_names.insert(from_lvl, name.id.to_string());
            }
        }

        Ok(Self {
            args_map,
            arg_var_names,
            metavar_ctx: metavar_ctx.clone(),
            meta_var: *meta_var,
            while_elaborating_span: *while_elaborating_span,
        })
    }
}

/// Extract the variable from an expression.
///
/// This function helps ensure condition 1 of Miller's pattern fragment,
/// in particular that the arguments of a metavariable consist of bound variables rather than arbitrary expressions.
/// However, in Polarity, variables can also occur underneath type annotations or as the solution of a hole.
/// Therefore, this function strips away these layers to get to the variable, if any.
///
/// # Parameters
///
/// - `exp`: The expression to extract the variable from
///
/// # Returns
///
/// `Some(variable)` if `exp` is a variable, or has a variable as a subexpression to layers of holes or type annotations.
///
fn expect_variable(exp: &Exp) -> Option<&Variable> {
    match exp {
        Exp::Variable(v) => Some(v),
        Exp::Hole(Hole { solution, .. }) => solution.as_ref().and_then(|sol| expect_variable(sol)),
        Exp::Anno(Anno { exp, .. }) => expect_variable(exp),
        _ => None,
    }
}

impl Shift for PartialRenaming {
    fn shift_in_range<R: ShiftRange>(&mut self, _range: &R, _by: (isize, isize)) {
        // PartialRenaming is shift-invariant, so nothing to do here
    }
}

impl Substitution for PartialRenaming {
    type Err = TypeError;

    fn get_subst(&self, ctx: &LevelCtx, lvl: Lvl) -> Result<Option<Box<Exp>>, Self::Err> {
        let from_binder = ctx.lookup(lvl);

        let Some(target_lvls) = self.args_map.get(&lvl) else {
            // Condition 2: every free variable of `candidate` occurs in `args`
            return Err(TypeError::MetaEquatedToOutOfScope {
                span: self.meta_var.span.to_miette(),
                meta_var: self.meta_var.print_to_string(None),
                out_of_scope: from_binder.name.to_string(),
                while_elaborating_span: self.while_elaborating_span.to_miette(),
            });
        };

        if target_lvls.len() > 1 {
            // Condition 1: `args` consists of *distinct* bound variables
            return Err(TypeError::MetaArgNotDistinct {
                span: self.meta_var.span.to_miette(),
                meta_var: self.meta_var.print_to_string(None),
                arg: self.arg_var_names[&lvl].clone(),
                while_elaborating_span: self.while_elaborating_span.to_miette(),
            });
        };
        assert_eq!(target_lvls.len(), 1);
        let target_lvl = target_lvls.iter().next().unwrap();
        let idx = self.metavar_ctx.lvl_to_idx(*target_lvl);
        let to_binder = self.metavar_ctx.lookup(*target_lvl);

        Ok(Some(Box::new(Exp::Variable(Variable {
            span: None,
            idx,
            name: match to_binder.name {
                VarBind::Var { id, .. } => VarBound { span: None, id },
                // When we encouter a wildcard, we use `x` as a placeholder name for the variable referencing this binder.
                // Of course, `x` is not guaranteed to be unique; in general we do not guarantee that the string representation of variables remains intact during elaboration.
                // When reliable variable names are needed (e.g. for printing source code or code generation), the `renaming` transformation needs to be applied to the AST first.
                ast::VarBind::Wildcard { .. } => VarBound::from_string("x"),
            },
            inferred_type: None,
        }))))
    }
}

fn zip_cases_by_xtors(
    cases_lhs: &[Case],
    cases_rhs: &[Case],
) -> impl Iterator<Item = (Case, Case)> {
    assert_eq!(cases_lhs.len(), cases_rhs.len());

    let mut cases = vec![];
    for case_lhs in cases_lhs {
        let rhs_index =
            cases_rhs.iter().position(|case_rhs| case_lhs.pattern.name == case_rhs.pattern.name);
        if let Some(rhs_index) = rhs_index {
            cases.push((case_lhs.clone(), cases_rhs[rhs_index].clone()));
        }
    }
    cases.into_iter()
}

#[cfg(test)]
mod tests {
    use ctx::LevelCtx;
    use url::Url;

    use super::*;

    fn var(name: &str, idx: (usize, usize)) -> Box<Exp> {
        Box::new(Exp::Variable(Variable {
            span: None,
            idx: Idx { fst: idx.0, snd: idx.1 },
            name: VarBound::from_string(name),
            inferred_type: None,
        }))
    }

    fn level_ctx(rows: Vec<Vec<&str>>) -> LevelCtx {
        let ctx: Vec<Vec<VarBind>> = rows
            .into_iter()
            .map(|row| row.into_iter().map(VarBind::from_string).collect())
            .collect();
        ctx.into()
    }

    fn meta_var(id: u64) -> MetaVar {
        MetaVar { span: None, kind: MetaVarKind::MustSolve, id }
    }

    fn dummy_uri() -> Url {
        Url::parse("inmemory://scratch.pol").unwrap()
    }

    fn true_exp() -> Box<Exp> {
        let uri = dummy_uri();
        let name = IdBound { span: None, id: "T".to_owned(), uri };
        Box::new(Exp::TypCtor(TypCtor { span: None, name, args: Args { args: vec![] } }))
    }

    fn bool_type() -> Box<Exp> {
        let uri = dummy_uri();
        let name = IdBound { span: None, id: "Bool".to_owned(), uri };
        Box::new(Exp::TypCtor(TypCtor { span: None, name, args: Args { args: vec![] } }))
    }

    fn fun_type(a: Box<Exp>, b: Box<Exp>) -> Box<Exp> {
        let uri = dummy_uri();
        let name = IdBound { span: None, id: "Fun".to_owned(), uri };
        Box::new(Exp::TypCtor(TypCtor {
            span: None,
            name,
            args: Args {
                args: vec![
                    Arg::UnnamedArg { arg: a, erased: false },
                    Arg::UnnamedArg { arg: b, erased: false },
                ],
            },
        }))
    }

    /// Violation of condition 1: The args contain a non-variable argument.
    ///
    /// Example problem: `?0(T) =? T`
    /// ```
    #[test]
    fn test_fail_meta_arg_not_variable() {
        use crate::result::TypeError::MetaArgNotVariable;

        let mut ctx = Ctx { constraints: vec![], done: HashSet::default() };
        let mut meta_vars = HashMap::default();

        let metavar_ctx = level_ctx(vec![vec!["x"]]);
        let metavar = meta_var(0);

        let args = &[vec![Binder { name: VarBind::from_string("x"), content: true_exp() }]];

        let constraint_ctx = level_ctx(vec![vec!["x"]]);
        let candidate = var("x", (0, 0));

        let err = ctx
            .solve_meta_var(
                &mut meta_vars,
                metavar_ctx,
                metavar,
                args,
                constraint_ctx,
                candidate,
                &None,
            )
            .unwrap_err();

        assert_eq!(
            *err,
            MetaArgNotVariable {
                span: None,
                meta_var: "_0".to_owned(),
                arg: "x:=T".to_owned(),
                while_elaborating_span: None,
            }
        );
    }

    /// Violation of condition 2: The candidate refers to a binder not in the metavariable’s arguments.
    ///
    /// Example problem: `?0(x,y) =? Fun(z, z)`
    #[test]
    fn test_fail_condition_2() {
        use crate::result::TypeError::MetaEquatedToOutOfScope;

        let mut ctx = Ctx { constraints: vec![], done: HashSet::default() };
        let mut meta_vars = HashMap::default();

        let metavar_ctx = level_ctx(vec![vec!["x", "y"]]);
        let metavar = meta_var(0);

        let args = &[vec![
            Binder { name: VarBind::from_string("x"), content: var("x", (0, 2)) },
            Binder { name: VarBind::from_string("y"), content: var("y", (0, 1)) },
        ]];

        let constraint_ctx = level_ctx(vec![vec!["x", "y", "z"]]);
        let candidate = var("z", (0, 0));

        let err = ctx
            .solve_meta_var(
                &mut meta_vars,
                metavar_ctx,
                metavar,
                args,
                constraint_ctx,
                candidate,
                &None,
            )
            .unwrap_err();

        assert_eq!(
            *err,
            MetaEquatedToOutOfScope {
                span: None,
                meta_var: "_0".to_owned(),
                out_of_scope: "z".to_owned(),
                while_elaborating_span: None,
            }
        );
    }

    /// Violation of condition 3: The candidate contains the metavariable itself.
    ///
    /// Example problem: `?0 =? Fun(?0, ?0)`
    #[test]
    fn test_fail_condition_3() {
        use crate::result::TypeError::MetaOccursCheckFailed;

        let mut ctx = Ctx { constraints: vec![], done: HashSet::default() };
        let mut meta_vars = HashMap::default();

        let metavar_ctx = level_ctx(vec![]);
        let metavar = meta_var(0);

        let args: &[Vec<Binder<Box<Exp>>>] = &[];

        let hole: Box<Exp> = Box::new(
            Hole {
                span: None,
                kind: MetaVarKind::MustSolve,
                metavar,
                inferred_type: None,
                inferred_ctx: None,
                args: vec![],
                solution: None,
            }
            .into(),
        );

        let candidate = fun_type(hole.clone(), hole);

        let err = ctx
            .solve_meta_var(
                &mut meta_vars,
                metavar_ctx,
                metavar,
                args,
                level_ctx(vec![vec![]]),
                candidate,
                &None,
            )
            .unwrap_err();

        assert_eq!(
            *err,
            MetaOccursCheckFailed {
                span: None,
                meta_var: "_0".to_owned(),
                while_elaborating_span: None
            }
        );
    }

    #[test]
    fn test_expect_variable_under_anno_and_hole() {
        let var_exp = var("x", (0, 0));

        let anno_and_hole_exp = Box::new(Exp::Anno(Anno {
            exp: Box::new(Exp::Hole(Hole {
                span: None,
                kind: MetaVarKind::MustSolve,
                metavar: meta_var(0),
                inferred_type: None,
                inferred_ctx: None,
                args: vec![],
                solution: Some(var_exp),
            })),
            span: None,
            typ: bool_type(),
            normalized_type: None,
        }));

        let result = expect_variable(&anno_and_hole_exp).unwrap();
        assert_eq!(
            result,
            &Variable {
                span: None,
                idx: Idx { fst: 0, snd: 0 },
                name: VarBound::from_string("x"),
                inferred_type: None
            }
        );
    }

    #[test]
    fn test_non_linear_arguments() {
        let mut ctx = Ctx { constraints: vec![], done: HashSet::default() };
        let mut meta_vars = HashMap::default();

        let metavar_ctx = level_ctx(vec![vec!["x", "y", "z"]]);
        let metavar = meta_var(0);

        let args = &[vec![
            Binder { name: VarBind::from_string("x"), content: var("a", (0, 1)) },
            Binder { name: VarBind::from_string("y"), content: var("a", (0, 1)) },
            Binder { name: VarBind::from_string("z"), content: var("c", (0, 0)) },
        ]];

        let constraint_ctx = level_ctx(vec![vec!["a", "c"]]);
        let candidate = var("c", (0, 0));

        ctx.solve_meta_var(
            &mut meta_vars,
            metavar_ctx.clone(),
            metavar,
            args,
            constraint_ctx,
            candidate,
            &None,
        )
        .unwrap();

        let solution = &meta_vars[&metavar];

        assert_eq!(solution, &MetaVarState::Solved { ctx: metavar_ctx, solution: var("z", (0, 0)) })
    }
}
