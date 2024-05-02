use std::rc::Rc;

use syntax::ctx::LevelCtx;
use syntax::generic::{occurs_in, Variable};

use crate::result::TypeError;
use crate::unifier::dec::{Dec, No, Yes};
use printer::{DocAllocator, Print};
use syntax::common::*;
use syntax::ust;

#[derive(Debug, Clone)]
pub struct Unificator {
    map: HashMap<Lvl, Rc<ust::Exp>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Eqn {
    pub lhs: Rc<ust::Exp>,
    pub rhs: Rc<ust::Exp>,
}

impl From<(Rc<ust::Exp>, Rc<ust::Exp>)> for Eqn {
    fn from((lhs, rhs): (Rc<ust::Exp>, Rc<ust::Exp>)) -> Self {
        Eqn { lhs, rhs }
    }
}

impl Shift for Eqn {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let Eqn { lhs, rhs } = self;
        Eqn { lhs: lhs.shift_in_range(range.clone(), by), rhs: rhs.shift_in_range(range, by) }
    }
}

impl Substitutable<Rc<ust::Exp>> for Unificator {
    fn subst<S: Substitution<Rc<ust::Exp>>>(&self, ctx: &mut LevelCtx, by: &S) -> Self {
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

impl Substitution<Rc<ust::Exp>> for Unificator {
    fn get_subst(&self, _ctx: &LevelCtx, lvl: Lvl) -> Option<Rc<ust::Exp>> {
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
    eqns: Vec<Eqn>,
    vars_are_rigid: bool,
) -> Result<Dec<Unificator>, TypeError> {
    let mut ctx = Ctx::new(eqns.clone(), ctx.clone(), vars_are_rigid);
    let res = match ctx.unify()? {
        Yes(_) => Yes(ctx.unif),
        No(()) => No(()),
    };
    Ok(res)
}

struct Ctx {
    /// Constraints that have not yet been solved
    eqns: Vec<Eqn>,
    /// A cache of solved constraints. We can skip solving a constraint
    /// if we have seen it before
    done: HashSet<Eqn>,
    ctx: LevelCtx,
    /// Partial solution that we have computed from solving previous constraints.
    unif: Unificator,
    /// When we use the unifier as a conversion checker then we don't want to
    /// treat two distinct variables as unifiable. In that case we call the unifier
    /// and enable this boolean flag in order to treat all variables as rigid.
    vars_are_rigid: bool,
}

impl Ctx {
    fn new(eqns: Vec<Eqn>, ctx: LevelCtx, vars_are_rigid: bool) -> Self {
        Self { eqns, done: HashSet::default(), ctx, unif: Unificator::empty(), vars_are_rigid }
    }

    fn unify(&mut self) -> Result<Dec, TypeError> {
        while let Some(eqn) = self.eqns.pop() {
            match self.unify_eqn(&eqn)? {
                Yes(_) => {
                    self.done.insert(eqn);
                }
                No(_) => return Ok(No(())),
            }
        }

        Ok(Yes(()))
    }

    fn unify_eqn(&mut self, eqn: &Eqn) -> Result<Dec, TypeError> {
        let Eqn { lhs, rhs, .. } = eqn;

        match (&**lhs, &**rhs) {
            (
                ust::Exp::Variable(Variable { idx: idx_1, .. }),
                ust::Exp::Variable(Variable { idx: idx_2, .. }),
            ) => {
                if idx_1 == idx_2 {
                    Ok(Yes(()))
                } else if self.vars_are_rigid {
                    Ok(No(()))
                } else {
                    self.add_assignment(*idx_1, rhs.clone())
                }
            }
            (ust::Exp::Variable(Variable { idx, .. }), _) => {
                if self.vars_are_rigid {
                    Ok(No(()))
                } else {
                    self.add_assignment(*idx, rhs.clone())
                }
            }
            (_, ust::Exp::Variable(Variable { idx, .. })) => {
                if self.vars_are_rigid {
                    Ok(No(()))
                } else {
                    self.add_assignment(*idx, lhs.clone())
                }
            }
            (
                ust::Exp::TypCtor(ust::TypCtor { name, args, .. }),
                ust::Exp::TypCtor(ust::TypCtor { name: name2, args: args2, .. }),
            ) if name == name2 => self.unify_args(args, args2),
            (
                ust::Exp::TypCtor(ust::TypCtor { name, .. }),
                ust::Exp::TypCtor(ust::TypCtor { name: name2, .. }),
            ) if name != name2 => Ok(No(())),
            (
                ust::Exp::Call(ust::Call { name, args, .. }),
                ust::Exp::Call(ust::Call { name: name2, args: args2, .. }),
            ) if name == name2 => self.unify_args(args, args2),
            (
                ust::Exp::Call(ust::Call { name, .. }),
                ust::Exp::Call(ust::Call { name: name2, .. }),
            ) if name != name2 => Ok(No(())),
            (
                ust::Exp::DotCall(ust::DotCall { exp, name, args, .. }),
                ust::Exp::DotCall(ust::DotCall { exp: exp2, name: name2, args: args2, .. }),
            ) if name == name2 => {
                self.add_equation(Eqn { lhs: exp.clone(), rhs: exp2.clone() })?;
                self.unify_args(args, args2)
            }
            (ust::Exp::TypeUniv(_), ust::Exp::TypeUniv(_)) => Ok(Yes(())),
            (ust::Exp::Anno(_), _) => Err(TypeError::unsupported_annotation(lhs.clone())),
            (_, ust::Exp::Anno(_)) => Err(TypeError::unsupported_annotation(rhs.clone())),
            (_, _) => Err(TypeError::cannot_decide(lhs.clone(), rhs.clone())),
        }
    }

    fn unify_args(&mut self, lhs: &ust::Args, rhs: &ust::Args) -> Result<Dec, TypeError> {
        let new_eqns = lhs.args.iter().cloned().zip(rhs.args.iter().cloned()).map(Eqn::from);
        self.add_equations(new_eqns)?;
        Ok(Yes(()))
    }

    fn add_assignment(&mut self, idx: Idx, exp: Rc<ust::Exp>) -> Result<Dec, TypeError> {
        if occurs_in(&mut self.ctx, idx, &exp) {
            return Err(TypeError::occurs_check_failed(idx, exp));
        }
        let insert_lvl = self.ctx.idx_to_lvl(idx);
        let exp = exp.subst(&mut self.ctx, &self.unif);
        self.unif = self.unif.subst(&mut self.ctx, &Assign(insert_lvl, exp.clone()));
        match self.unif.map.get(&insert_lvl) {
            Some(other_exp) => {
                let eqn = Eqn { lhs: exp, rhs: other_exp.clone() };
                self.add_equation(eqn)
            }
            None => {
                self.unif.map.insert(insert_lvl, exp);
                Ok(Yes(()))
            }
        }
    }

    fn add_equation(&mut self, eqn: Eqn) -> Result<Dec, TypeError> {
        self.add_equations([eqn])
    }

    fn add_equations<I: IntoIterator<Item = Eqn>>(&mut self, iter: I) -> Result<Dec, TypeError> {
        self.eqns.extend(iter.into_iter().filter(|eqn| !self.done.contains(eqn)));
        Ok(Yes(()))
    }
}

impl<'a> Print<'a> for Eqn {
    fn print(
        &'a self,
        cfg: &printer::PrintCfg,
        alloc: &'a printer::Alloc<'a>,
    ) -> printer::Builder<'a> {
        self.lhs.print(cfg, alloc).append(" = ").append(self.rhs.print(cfg, alloc))
    }
}

impl<'a> Print<'a> for Unificator {
    fn print(
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
