use std::collections::HashSet;

use ast::Variable;
use ctx::GenericCtx;
use miette_util::codespan::Span;

use crate::result::{TcResult, TypeError};
use ast::*;
use printer::Print;

use super::constraints::Constraint;
use super::dec::{Dec, No, Yes};

pub struct Ctx {
    /// Constraints that have not yet been solved
    pub constraints: Vec<Constraint>,
    /// A cache of solved constraints. We can skip solving a constraint
    /// if we have seen it before
    pub done: HashSet<Constraint>,
}

/// Tests whether the hole is in Miller's pattern fragment, i.e. whether it is applied
/// to distinct bound variables.
fn is_solvable(h: &Hole) -> bool {
    let Hole { args, .. } = h;
    let mut seen: HashSet<Idx> = HashSet::new();
    for subst in args {
        for exp in subst {
            let Exp::Variable(v) = &**exp else {
                return false;
            };
            if seen.contains(&v.idx) {
                return false;
            } else {
                seen.insert(v.idx);
                continue;
            }
        }
    }
    true
}

impl Ctx {
    pub fn new(constraints: Vec<Constraint>) -> Self {
        Self { constraints, done: HashSet::default() }
    }

    pub fn unify(
        &mut self,
        meta_vars: &mut HashMap<MetaVar, MetaVarState>,
        while_elaborating_span: &Option<Span>,
    ) -> TcResult<Dec> {
        while let Some(constraint) = self.constraints.pop() {
            match self.unify_eqn(&constraint, meta_vars, while_elaborating_span)? {
                Yes => {
                    self.done.insert(constraint);
                }
                No => return Ok(No),
            }
        }

        Ok(Yes)
    }

    fn unify_eqn(
        &mut self,
        eqn: &Constraint,
        meta_vars: &mut HashMap<MetaVar, MetaVarState>,
        while_elaborating_span: &Option<Span>,
    ) -> TcResult<Dec> {
        match eqn {
            Constraint::Equality { ctx: constraint_cxt, lhs, rhs } => match (&**lhs, &**rhs) {
                (Exp::Hole(h), e) | (e, Exp::Hole(h)) => {
                    let metavar_state = meta_vars.get(&h.metavar).unwrap();
                    match metavar_state {
                        MetaVarState::Solved { ctx, solution } => {
                            let lhs = solution.clone().subst(&mut ctx.clone(), &h.args);
                            self.add_constraint(Constraint::Equality {
                                ctx: constraint_cxt.clone(),
                                lhs,
                                rhs: Box::new(e.clone()),
                            })?;
                        }
                        MetaVarState::Unsolved { ctx } => {
                            if is_solvable(h) {
                                self.solve_meta_var(
                                    meta_vars,
                                    h.metavar,
                                    ctx.clone(),
                                    Box::new(e.clone()),
                                )?;
                            } else {
                                return Err(TypeError::cannot_decide(
                                    &Box::new(Exp::Hole(h.clone())),
                                    &Box::new(e.clone()),
                                    while_elaborating_span,
                                ));
                            }
                        }
                    }

                    Ok(Yes)
                }
                (
                    Exp::Variable(Variable { idx: idx_1, .. }),
                    Exp::Variable(Variable { idx: idx_2, .. }),
                ) => {
                    if idx_1 == idx_2 {
                        Ok(Yes)
                    } else {
                        Ok(No)
                    }
                }
                (Exp::Variable(Variable { .. }), _) => Ok(No),
                (_, Exp::Variable(Variable { .. })) => Ok(No),
                (
                    Exp::TypCtor(TypCtor { name, args, .. }),
                    Exp::TypCtor(TypCtor { name: name2, args: args2, .. }),
                ) if name == name2 => {
                    let constraint = Constraint::EqualityArgs {
                        ctx: constraint_cxt.clone(),
                        lhs: args.clone(),
                        rhs: args2.clone(),
                    };
                    self.add_constraint(constraint)
                }
                (Exp::TypCtor(TypCtor { name, .. }), Exp::TypCtor(TypCtor { name: name2, .. }))
                    if name != name2 =>
                {
                    Ok(No)
                }
                (
                    Exp::Call(Call { name, args, .. }),
                    Exp::Call(Call { name: name2, args: args2, .. }),
                ) if name == name2 => {
                    let constraint = Constraint::EqualityArgs {
                        ctx: constraint_cxt.clone(),
                        lhs: args.clone(),
                        rhs: args2.clone(),
                    };
                    self.add_constraint(constraint)
                }
                (Exp::Call(Call { name, .. }), Exp::Call(Call { name: name2, .. }))
                    if name != name2 =>
                {
                    Ok(No)
                }
                (
                    Exp::DotCall(DotCall { exp, name, args, .. }),
                    Exp::DotCall(DotCall { exp: exp2, name: name2, args: args2, .. }),
                ) if name == name2 => {
                    self.add_constraint(Constraint::Equality {
                        ctx: constraint_cxt.clone(),
                        lhs: exp.clone(),
                        rhs: exp2.clone(),
                    })?;
                    let constraint = Constraint::EqualityArgs {
                        ctx: constraint_cxt.clone(),
                        lhs: args.clone(),
                        rhs: args2.clone(),
                    };
                    self.add_constraint(constraint)
                }
                (Exp::TypeUniv(_), Exp::TypeUniv(_)) => Ok(Yes),
                (Exp::Anno(Anno { exp, .. }), rhs) => self.add_constraint(Constraint::Equality {
                    ctx: constraint_cxt.clone(),
                    lhs: exp.clone(),
                    rhs: Box::new(rhs.clone()),
                }),
                (lhs, Exp::Anno(Anno { exp, .. })) => self.add_constraint(Constraint::Equality {
                    ctx: constraint_cxt.clone(),
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
                                Some(Constraint::Equality { ctx: constraint_cxt.clone(), lhs, rhs })
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
                            ctx: constraint_ctx.clone(),
                            lhs: lhs.exp().clone(),
                            rhs: rhs.exp().clone(),
                        }
                    });
                self.add_constraints(new_eqns)?;
                Ok(Yes)
            }
        }
    }

    fn add_constraint(&mut self, eqn: Constraint) -> TcResult<Dec> {
        self.add_constraints([eqn])
    }

    fn add_constraints<I: IntoIterator<Item = Constraint>>(&mut self, iter: I) -> TcResult<Dec> {
        self.constraints.extend(iter.into_iter().filter(|eqn| !self.done.contains(eqn)));
        Ok(Yes)
    }

    fn solve_meta_var(
        &mut self,
        meta_vars: &mut HashMap<MetaVar, MetaVarState>,
        metavar: MetaVar,
        ctx: GenericCtx<()>,
        mut solution: Box<Exp>,
    ) -> TcResult {
        log::trace!(
            "Solved metavariable: {} with solution: {}",
            metavar.id,
            solution.print_trace()
        );
        solution
            .zonk(meta_vars)
            .map_err(|err| TypeError::Impossible { message: err.to_string(), span: None })?;
        meta_vars.insert(metavar, MetaVarState::Solved { ctx: ctx.clone(), solution });
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
