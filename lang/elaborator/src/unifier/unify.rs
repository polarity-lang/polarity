use std::collections::HashSet;
use std::rc::Rc;

use syntax::ast::{occurs_in, Variable};
use syntax::ctx::LevelCtx;

use crate::result::TypeError;
use crate::unifier::dec::{Dec, No, Yes};
use printer::{DocAllocator, Print};
use syntax::ast::*;

use super::constraints::Constraint;

#[derive(Debug, Clone)]
pub struct Unificator {
    map: HashMap<Lvl, Rc<Exp>>,
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
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        Self {
            map: self
                .map
                .iter()
                .map(|(lvl, exp)| (*lvl, exp.shift_in_range(range.clone(), by)))
                .collect(),
        }
    }
}

impl Substitution for Unificator {
    fn get_subst(&self, _ctx: &LevelCtx, lvl: Lvl) -> Option<Rc<Exp>> {
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
    meta_vars: &mut HashMap<MetaVar, MetaVarState>,
    constraint: Constraint,
    vars_are_rigid: bool,
) -> Result<Dec<Unificator>, TypeError> {
    let mut ctx = Ctx::new(vec![constraint], ctx.clone(), vars_are_rigid);
    let res = match ctx.unify(meta_vars)? {
        Yes(_) => Yes(ctx.unif),
        No(()) => No(()),
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
    /// When we use the unifier as a conversion checker then we don't want to
    /// treat two distinct variables as unifiable. In that case we call the unifier
    /// and enable this boolean flag in order to treat all variables as rigid.
    vars_are_rigid: bool,
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
    fn new(constraints: Vec<Constraint>, ctx: LevelCtx, vars_are_rigid: bool) -> Self {
        Self {
            constraints,
            done: HashSet::default(),
            ctx,
            unif: Unificator::empty(),
            vars_are_rigid,
        }
    }

    fn unify(&mut self, meta_vars: &mut HashMap<MetaVar, MetaVarState>) -> Result<Dec, TypeError> {
        while let Some(constraint) = self.constraints.pop() {
            match self.unify_eqn(&constraint, meta_vars)? {
                Yes(_) => {
                    self.done.insert(constraint);
                }
                No(_) => return Ok(No(())),
            }
        }

        Ok(Yes(()))
    }

    fn unify_eqn(
        &mut self,
        eqn: &Constraint,
        meta_vars: &mut HashMap<MetaVar, MetaVarState>,
    ) -> Result<Dec, TypeError> {
        match eqn {
            Constraint::Equality { lhs, rhs, .. } => match (&**lhs, &**rhs) {
                (Exp::Hole(h), e) | (e, Exp::Hole(h)) if self.vars_are_rigid => {
                    let metavar_state = meta_vars.get(&h.metavar).unwrap();
                    match metavar_state {
                        MetaVarState::Solved { ctx, solution } => {
                            let lhs = solution.clone().subst(&mut ctx.clone(), &h.args);
                            self.add_constraint(Constraint::Equality {
                                lhs,
                                rhs: Rc::new(e.clone()),
                            })?;
                        }
                        MetaVarState::Unsolved { ctx } => {
                            if is_solvable(h) {
                                meta_vars.insert(
                                    h.metavar,
                                    MetaVarState::Solved {
                                        ctx: ctx.clone(),
                                        solution: Rc::new(e.clone()),
                                    },
                                );
                            } else {
                                return Err(TypeError::cannot_decide(
                                    Rc::new(Exp::Hole(h.clone())),
                                    Rc::new(e.clone()),
                                ));
                            }
                        }
                    }

                    Ok(Yes(()))
                }
                (
                    Exp::Variable(Variable { idx: idx_1, .. }),
                    Exp::Variable(Variable { idx: idx_2, .. }),
                ) => {
                    if idx_1 == idx_2 {
                        Ok(Yes(()))
                    } else if self.vars_are_rigid {
                        Ok(No(()))
                    } else {
                        self.add_assignment(*idx_1, rhs.clone())
                    }
                }
                (Exp::Variable(Variable { idx, .. }), _) => {
                    if self.vars_are_rigid {
                        Ok(No(()))
                    } else {
                        self.add_assignment(*idx, rhs.clone())
                    }
                }
                (_, Exp::Variable(Variable { idx, .. })) => {
                    if self.vars_are_rigid {
                        Ok(No(()))
                    } else {
                        self.add_assignment(*idx, lhs.clone())
                    }
                }
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
                    Ok(No(()))
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
                    Ok(No(()))
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
                (Exp::Anno(_), _) => Err(TypeError::unsupported_annotation(lhs.clone())),
                (_, Exp::Anno(_)) => Err(TypeError::unsupported_annotation(rhs.clone())),
                (_, _) => Err(TypeError::cannot_decide(lhs.clone(), rhs.clone())),
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

    fn add_assignment(&mut self, idx: Idx, exp: Rc<Exp>) -> Result<Dec, TypeError> {
        if occurs_in(&mut self.ctx, idx, &exp) {
            return Err(TypeError::occurs_check_failed(idx, exp));
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

    fn add_constraint(&mut self, eqn: Constraint) -> Result<Dec, TypeError> {
        self.add_constraints([eqn])
    }

    fn add_constraints<I: IntoIterator<Item = Constraint>>(
        &mut self,
        iter: I,
    ) -> Result<Dec, TypeError> {
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
