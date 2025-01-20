use std::collections::HashSet;

use ast::ctx::LevelCtx;
use ast::{occurs_in, Variable};
use miette_util::codespan::Span;

use crate::result::{TcResult, TypeError};
use ast::*;
use printer::{DocAllocator, Print};

use super::constraints::Constraint;
use super::dec::{Dec, No, Yes};

#[derive(Debug, Clone)]
pub struct Unificator {
    map: HashMap<Lvl, Box<Exp>>,
}

impl Substitutable for Unificator {
    type Result = Unificator;
    fn subst<S: Substitution>(&self, ctx: &mut LevelCtx, by: &S) -> Self {
        let map = self
            .map
            .iter()
            .map(|(entry_lvl, entry_val)| (*entry_lvl, entry_val.subst(ctx, by)))
            .collect();
        Self { map }
    }
}

impl Shift for Unificator {
    fn shift_in_range<R: ShiftRange>(&mut self, range: &R, by: (isize, isize)) {
        self.map.iter_mut().for_each(|(_, exp)| exp.shift_in_range(range, by));
    }
}

impl Substitution for Unificator {
    fn get_subst(&self, _ctx: &LevelCtx, lvl: Lvl) -> Option<Box<Exp>> {
        self.map.get(&lvl).cloned()
    }
}

impl Unificator {
    pub fn empty() -> Self {
        Self { map: HashMap::default() }
    }
}

pub fn unify(
    ctx: LevelCtx,
    constraint: Constraint,
    while_elaborating_span: &Option<Span>,
) -> TcResult<Dec<Unificator>> {
    let mut ctx = Ctx::new(vec![constraint], ctx.clone());
    let res = match ctx.unify(while_elaborating_span)? {
        Yes(_) => Yes(ctx.unif),
        No => No,
    };
    Ok(res)
}

struct Ctx {
    /// Constraints that have not yet been solved
    constraints: Vec<Constraint>,
    /// A cache of solved constraints. We can skip solving a constraint
    /// if we have seen it before
    done: HashSet<Constraint>,
    ctx: LevelCtx,
    /// Partial solution that we have computed from solving previous constraints.
    unif: Unificator,
}

impl Ctx {
    fn new(constraints: Vec<Constraint>, ctx: LevelCtx) -> Self {
        Self { constraints, done: HashSet::default(), ctx, unif: Unificator::empty() }
    }

    fn unify(&mut self, while_elaborating_span: &Option<Span>) -> TcResult<Dec> {
        while let Some(constraint) = self.constraints.pop() {
            match self.unify_eqn(&constraint, while_elaborating_span)? {
                Yes(_) => {
                    self.done.insert(constraint);
                }
                No => return Ok(No),
            }
        }

        Ok(Yes(()))
    }

    fn unify_eqn(
        &mut self,
        eqn: &Constraint,
        while_elaborating_span: &Option<Span>,
    ) -> TcResult<Dec> {
        match eqn {
            Constraint::Equality { lhs, rhs, .. } => match (&**lhs, &**rhs) {
                (
                    Exp::Variable(Variable { idx: idx_1, .. }),
                    Exp::Variable(Variable { idx: idx_2, .. }),
                ) => {
                    if idx_1 == idx_2 {
                        Ok(Yes(()))
                    } else {
                        self.add_assignment(*idx_1, rhs.clone())
                    }
                }
                (Exp::Variable(Variable { idx, .. }), _) => self.add_assignment(*idx, rhs.clone()),
                (_, Exp::Variable(Variable { idx, .. })) => self.add_assignment(*idx, lhs.clone()),
                (
                    Exp::TypCtor(TypCtor { name, args, .. }),
                    Exp::TypCtor(TypCtor { name: name2, args: args2, .. }),
                ) if name == name2 => {
                    let constraint =
                        Constraint::EqualityArgs { lhs: args.clone(), rhs: args2.clone() };
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
                    let constraint =
                        Constraint::EqualityArgs { lhs: args.clone(), rhs: args2.clone() };
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
                        lhs: exp.clone(),
                        rhs: exp2.clone(),
                    })?;
                    let constraint =
                        Constraint::EqualityArgs { lhs: args.clone(), rhs: args2.clone() };
                    self.add_constraint(constraint)
                }
                (Exp::TypeUniv(_), Exp::TypeUniv(_)) => Ok(Yes(())),
                (Exp::Anno(Anno { exp, .. }), rhs) => self.add_constraint(Constraint::Equality {
                    lhs: exp.clone(),
                    rhs: Box::new(rhs.clone()),
                }),
                (lhs, Exp::Anno(Anno { exp, .. })) => self.add_constraint(Constraint::Equality {
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
                                Some(Constraint::Equality { lhs, rhs })
                            } else {
                                None
                            }
                        });
                    self.add_constraints(new_eqns)
                }
                (_, _) => Err(TypeError::cannot_decide(lhs, rhs, while_elaborating_span)),
            },
            Constraint::EqualityArgs { lhs, rhs } => {
                let new_eqns =
                    lhs.args.iter().cloned().zip(rhs.args.iter().cloned()).map(|(lhs, rhs)| {
                        Constraint::Equality { lhs: lhs.exp().clone(), rhs: rhs.exp().clone() }
                    });
                self.add_constraints(new_eqns)?;
                Ok(Yes(()))
            }
        }
    }

    fn add_assignment(&mut self, idx: Idx, exp: Box<Exp>) -> TcResult<Dec> {
        if occurs_in(&mut self.ctx, idx, &exp) {
            return Err(TypeError::occurs_check_failed(idx, &exp));
        }
        let insert_lvl = self.ctx.idx_to_lvl(idx);
        let exp = exp.subst(&mut self.ctx, &self.unif);
        self.unif = self.unif.subst(&mut self.ctx, &Assign { lvl: insert_lvl, exp: exp.clone() });
        match self.unif.map.get(&insert_lvl) {
            Some(other_exp) => {
                let eqn = Constraint::Equality { lhs: exp, rhs: other_exp.clone() };
                self.add_constraint(eqn)
            }
            None => {
                self.unif.map.insert(insert_lvl, exp);
                Ok(Yes(()))
            }
        }
    }

    fn add_constraint(&mut self, eqn: Constraint) -> TcResult<Dec> {
        self.add_constraints([eqn])
    }

    fn add_constraints<I: IntoIterator<Item = Constraint>>(&mut self, iter: I) -> TcResult<Dec> {
        self.constraints.extend(iter.into_iter().filter(|eqn| !self.done.contains(eqn)));
        Ok(Yes(()))
    }
}

impl Print for Unificator {
    fn print<'a>(
        &'a self,
        cfg: &printer::PrintCfg,
        alloc: &'a printer::Alloc<'a>,
    ) -> printer::Builder<'a> {
        let mut keys: Vec<_> = self.map.keys().collect();
        keys.sort();
        let exps = keys.into_iter().map(|key| {
            alloc.text(format!("{key}")).append(" := ").append(self.map[key].print(cfg, alloc))
        });
        alloc.intersperse(exps, ",").enclose("{", "}")
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
