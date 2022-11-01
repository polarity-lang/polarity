use std::rc::Rc;

use miette::Diagnostic;
use thiserror::Error;

use data::{Dec, HashMap, HashSet, No, Yes};
use printer::PrintToString;
use syntax::ast::{self, Assign, Substitutable, Substitution};
use syntax::de_bruijn::*;
use syntax::equiv::AlphaEq;
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

impl Substitutable for Unificator {
    fn subst<L: Leveled, S: Substitution>(&self, lvl: &L, by: &S) -> Self {
        let map = self
            .map
            .iter()
            .map(|(entry_lvl, entry_val)| (*entry_lvl, entry_val.subst(lvl, by)))
            .collect();
        Self { map }
    }
}

impl Substitution for Unificator {
    fn get(&self, lvl: Lvl) -> Option<Rc<ust::Exp>> {
        self.map.get(&lvl).cloned()
    }
}

impl Unificator {
    pub fn empty() -> Self {
        Self { map: HashMap::default() }
    }
}

#[trace("unify({:?} ) ~> {return:?}", eqns, data::id)]
pub fn unify<L: Leveled>(lvl: &L, eqns: Vec<Eqn>) -> Result<Dec<Unificator>, UnifyError> {
    let mut ctx = Ctx::new(eqns.clone(), lvl);
    let res = match ctx.unify()? {
        Yes(_) => Yes(ctx.unif),
        No(()) => No(()),
    };
    Ok(res)
}

struct Ctx<'l, L: Leveled> {
    eqns: Vec<Eqn>,
    done: HashSet<Eqn>,
    lvl: &'l L,
    unif: Unificator,
}

impl<'l, L: Leveled> Ctx<'l, L> {
    fn new(eqns: Vec<Eqn>, lvl: &'l L) -> Self {
        Self { eqns, done: HashSet::default(), lvl, unif: Unificator::empty() }
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
        if ast::occurs_in(self.lvl, idx, &exp) {
            return Err(UnifyError::occurs_check_failed(idx, exp));
        }
        let insert_lvl = self.lvl.idx_to_lvl(idx);
        let exp = exp.subst(self.lvl, &self.unif);
        self.unif.subst(self.lvl, &Assign(insert_lvl, &exp));
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
