use std::rc::Rc;

use miette::Diagnostic;
use syntax::ctx::LevelCtx;
use thiserror::Error;

use data::{Dec, HashMap, HashSet, No, Yes};
use printer::{DocAllocator, Print, PrintToString};
use syntax::ast;
use syntax::common::*;
use syntax::nf;
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

impl From<(Rc<nf::Nf>, Rc<nf::Nf>)> for Eqn {
    fn from((lhs, rhs): (Rc<nf::Nf>, Rc<nf::Nf>)) -> Self {
        Eqn { lhs: lhs.forget(), rhs: rhs.forget() }
    }
}

impl From<(Rc<ust::Exp>, Rc<ust::Exp>)> for Eqn {
    fn from((lhs, rhs): (Rc<ust::Exp>, Rc<ust::Exp>)) -> Self {
        Eqn { lhs, rhs }
    }
}

impl ShiftInRange for Eqn {
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

impl ShiftInRange for Unificator {
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

#[trace("{} |- unify({:P}) ~> {return:P}", ctx, eqns, data::id)]
pub fn unify(ctx: LevelCtx, eqns: Vec<Eqn>) -> Result<Dec<Unificator>, UnifyError> {
    let mut ctx = Ctx::new(eqns.clone(), ctx.clone());
    let res = match ctx.unify()? {
        Yes(_) => Yes(ctx.unif),
        No(()) => No(()),
    };
    Ok(res)
}

struct Ctx {
    eqns: Vec<Eqn>,
    done: HashSet<Eqn>,
    ctx: LevelCtx,
    unif: Unificator,
}

impl Ctx {
    fn new(eqns: Vec<Eqn>, ctx: LevelCtx) -> Self {
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
        self.unif.subst(&mut self.ctx, &Assign(insert_lvl, exp.clone()));
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
