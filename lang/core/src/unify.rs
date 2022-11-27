use std::rc::Rc;

use miette::Diagnostic;
use thiserror::Error;

use data::{Dec, HashMap, HashSet, No, Yes};
use printer::PrintToString;
use syntax::ast::{self, subst, Assign, Substitutable, Substitution};
use syntax::common::*;
use syntax::ust;
use tracer::trace;

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

impl ShiftCutoff for Eqn {
    fn shift_cutoff(&self, cutoff: usize, by: (isize, isize)) -> Self {
        let Eqn { lhs, rhs } = self;
        Eqn { lhs: lhs.shift_cutoff(cutoff, by), rhs: rhs.shift_cutoff(cutoff, by) }
    }
}

impl Substitutable<ust::UST> for Unificator {
    fn subst<S: Substitution<ust::UST>>(&self, ctx: &mut subst::Ctx, by: &S) -> Self {
        let map = self
            .map
            .iter()
            .map(|(entry_lvl, entry_val)| (*entry_lvl, entry_val.subst(ctx, by)))
            .collect();
        Self { map }
    }
}

impl Substitution<ust::UST> for Unificator {
    fn get_subst(&self, _ctx: &subst::Ctx, lvl: Lvl) -> Option<Rc<ust::Exp>> {
        self.map.get(&lvl).cloned()
    }
}

impl Unificator {
    pub fn empty() -> Self {
        Self { map: HashMap::default() }
    }
}

#[trace("unify({:?} ) ~> {return:?}", eqns, data::id)]
pub fn unify(ctx: subst::Ctx, eqns: Vec<Eqn>) -> Result<Dec<Unificator>, UnifyError> {
    let mut ctx = Ctx::new(eqns.clone(), ctx);
    let res = match ctx.unify()? {
        Yes(_) => Yes(ctx.unif),
        No(()) => No(()),
    };
    Ok(res)
}

struct Ctx {
    eqns: Vec<Eqn>,
    done: HashSet<Eqn>,
    ctx: subst::Ctx,
    unif: Unificator,
}

impl Ctx {
    fn new(eqns: Vec<Eqn>, ctx: subst::Ctx) -> Self {
        Self { eqns, done: HashSet::default(), ctx, unif: Unificator::empty() }
    }

    fn unify(&mut self) -> Result<Dec, UnifyError> {
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

    fn unify_eqn(&mut self, eqn: &Eqn) -> Result<Dec, UnifyError> {
        use ast::Exp::*;

        let Eqn { lhs, rhs, .. } = eqn;
        // FIXME: This is only temporary (not compatible with xfunc in general)
        if lhs.alpha_eq(rhs) {
            return Ok(Yes(()));
        }
        match (&**lhs, &**rhs) {
            (Var { idx, .. }, _) => self.add_assignment(*idx, rhs.clone()),
            (_, Var { idx, .. }) => self.add_assignment(*idx, lhs.clone()),
            (TypCtor { name, args: subst, .. }, TypCtor { name: name2, args: subst2, .. })
                if name == name2 =>
            {
                self.unify_args(subst, subst2)
            }
            (Ctor { name, args: subst, .. }, Ctor { name: name2, args: subst2, .. })
                if name == name2 =>
            {
                self.unify_args(subst, subst2)
            }
            (Ctor { name, .. }, Ctor { name: name2, .. }) if name != name2 => Ok(No(())),
            (
                Dtor { exp, name, args: subst, .. },
                Dtor { exp: exp2, name: name2, args: subst2, .. },
            ) if name == name2 => {
                self.add_equation(Eqn { lhs: exp.clone(), rhs: exp2.clone() })?;
                self.unify_args(subst, subst2)
            }
            (Type { .. }, Type { .. }) => Ok(Yes(())),
            (Anno { .. }, _) => Err(UnifyError::unsupported_annotation(lhs.clone())),
            (_, Anno { .. }) => Err(UnifyError::unsupported_annotation(rhs.clone())),
            (_, _) => Err(UnifyError::cannot_decide(lhs.clone(), rhs.clone())),
        }
    }

    fn unify_args(
        &mut self,
        lhs: &[Rc<ust::Exp>],
        rhs: &[Rc<ust::Exp>],
    ) -> Result<Dec, UnifyError> {
        let new_eqns = lhs.iter().cloned().zip(rhs.iter().cloned()).map(Eqn::from);
        self.add_equations(new_eqns)?;
        Ok(Yes(()))
    }

    fn add_assignment(&mut self, idx: Idx, exp: Rc<ust::Exp>) -> Result<Dec, UnifyError> {
        if ast::occurs_in(&mut self.ctx, idx, &exp) {
            return Err(UnifyError::occurs_check_failed(idx, exp));
        }
        let insert_lvl = self.ctx.idx_to_lvl(idx);
        let exp = exp.subst(&mut self.ctx, &self.unif);
        self.unif.subst(&mut self.ctx, &Assign(insert_lvl, &exp));
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

    fn add_equation(&mut self, eqn: Eqn) -> Result<Dec, UnifyError> {
        self.add_equations([eqn])
    }

    fn add_equations<I: IntoIterator<Item = Eqn>>(&mut self, iter: I) -> Result<Dec, UnifyError> {
        self.eqns.extend(iter.into_iter().filter(|eqn| !self.done.contains(eqn)));
        Ok(Yes(()))
    }
}

#[derive(Error, Diagnostic, Debug)]
pub enum UnifyError {
    #[error("{idx} occurs in {exp}")]
    OccursCheckFailed { idx: Idx, exp: String },
    #[error("Cannot unify annotated expression {exp}")]
    UnsupportedAnnotation { exp: String },
    #[error("Cannot automatically decide whether {lhs} and {rhs} unify")]
    CannotDecide { lhs: String, rhs: String },
}

impl UnifyError {
    fn occurs_check_failed(idx: Idx, exp: Rc<ust::Exp>) -> Self {
        Self::OccursCheckFailed { idx, exp: exp.print_to_string() }
    }

    fn unsupported_annotation(exp: Rc<ust::Exp>) -> Self {
        Self::UnsupportedAnnotation { exp: exp.print_to_string() }
    }

    fn cannot_decide(lhs: Rc<ust::Exp>, rhs: Rc<ust::Exp>) -> Self {
        Self::CannotDecide { lhs: lhs.print_to_string(), rhs: rhs.print_to_string() }
    }
}
