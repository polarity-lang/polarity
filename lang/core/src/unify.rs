use std::error::Error;
use std::fmt;
use std::rc::Rc;

use data::{Dec, HashMap, HashSet, No, Yes};
use printer::PrintToString;
use syntax::ast::{self, Assign};
use syntax::ast::{Substitutable, Substitution};
use syntax::de_bruijn::*;
use syntax::equiv::AlphaEq;
use tracer::trace;

#[derive(Debug, Clone)]
pub struct Unificator {
    map: HashMap<Lvl, Rc<ast::Exp>>,
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
    fn get(&self, lvl: Lvl) -> Option<Rc<ast::Exp>> {
        self.map.get(&lvl).cloned()
    }
}

impl Unificator {
    pub fn empty() -> Self {
        Self { map: HashMap::default() }
    }
}

#[trace("unify({:P} ) ~> {return:?}", eqns, data::id)]
pub fn unify<L: Leveled>(lvl: &L, eqns: Vec<ast::Eqn>) -> Result<Dec<Unificator>, UnifyError> {
    let mut ctx = Ctx::new(eqns.clone(), lvl);
    let res = match ctx.unify()? {
        Yes(_) => Yes(ctx.unif),
        No(()) => No(()),
    };
    Ok(res)
}

struct Ctx<'l, L: Leveled> {
    eqns: Vec<ast::Eqn>,
    done: HashSet<ast::Eqn>,
    lvl: &'l L,
    unif: Unificator,
}

impl<'l, L: Leveled> Ctx<'l, L> {
    fn new(eqns: Vec<ast::Eqn>, lvl: &'l L) -> Self {
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

    fn unify_eqn(&mut self, eqn: &ast::Eqn) -> Result<Dec, UnifyError> {
        use ast::Exp::*;

        let ast::Eqn { lhs, rhs, .. } = eqn;
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
                self.add_equation(ast::Eqn {
                    info: ast::Info::empty(),
                    lhs: exp.clone(),
                    rhs: exp2.clone(),
                })?;
                self.unify_args(subst, subst2)
            }
            (Type { .. }, Type { .. }) => Ok(Yes(())),
            (Anno { .. }, _) => Err(UnifyError::UnsupportedAnnotation { exp: lhs.clone() }),
            (_, Anno { .. }) => Err(UnifyError::UnsupportedAnnotation { exp: lhs.clone() }),
            (_, _) => Err(UnifyError::StructurallyDifferent { lhs: lhs.clone(), rhs: rhs.clone() }),
        }
    }

    fn unify_args(
        &mut self,
        lhs: &[Rc<ast::Exp>],
        rhs: &[Rc<ast::Exp>],
    ) -> Result<Dec, UnifyError> {
        let new_eqns = lhs.iter().cloned().zip(rhs.iter().cloned()).map(ast::Eqn::from);
        self.add_equations(new_eqns)?;
        Ok(Yes(()))
    }

    fn add_assignment(&mut self, idx: Idx, exp: Rc<ast::Exp>) -> Result<Dec, UnifyError> {
        if ast::occurs_in(self.lvl, idx, &exp) {
            return Err(UnifyError::OccursCheckFailed { idx, exp });
        }
        let insert_lvl = self.lvl.idx_to_lvl(idx);
        let exp = exp.subst(self.lvl, &self.unif);
        self.unif.subst(self.lvl, &Assign(insert_lvl, &exp));
        match self.unif.map.get(&insert_lvl) {
            Some(other_exp) => {
                let eqn = ast::Eqn { info: ast::Info::empty(), lhs: exp, rhs: other_exp.clone() };
                self.add_equation(eqn)
            }
            None => {
                self.unif.map.insert(insert_lvl, exp);
                Ok(Yes(()))
            }
        }
    }

    fn add_equation(&mut self, eqn: ast::Eqn) -> Result<Dec, UnifyError> {
        self.add_equations([eqn])
    }

    fn add_equations<I: IntoIterator<Item = ast::Eqn>>(
        &mut self,
        iter: I,
    ) -> Result<Dec, UnifyError> {
        self.eqns.extend(iter.into_iter().filter(|eqn| !self.done.contains(eqn)));
        Ok(Yes(()))
    }
}

#[derive(Debug)]
pub enum UnifyError {
    OccursCheckFailed { idx: Idx, exp: Rc<ast::Exp> },
    UnsupportedAnnotation { exp: Rc<ast::Exp> },
    StructurallyDifferent { lhs: Rc<ast::Exp>, rhs: Rc<ast::Exp> },
}

impl Error for UnifyError {}

impl fmt::Display for UnifyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnifyError::OccursCheckFailed { idx, exp } => {
                write!(f, "{} occurs in {}", idx, exp.print_to_string())
            }
            UnifyError::UnsupportedAnnotation { exp } => {
                write!(f, "Cannot unify annotated expression {}", exp.print_to_string())
            }
            UnifyError::StructurallyDifferent { lhs, rhs } => write!(
                f,
                "Cannot unify {} and {} because they are structurally different",
                lhs.print_to_string(),
                rhs.print_to_string()
            ),
        }
    }
}
