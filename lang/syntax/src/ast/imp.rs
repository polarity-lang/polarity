use crate::de_bruijn::*;
use crate::equiv::*;

use super::def::*;

impl ShiftCutoff for Eqn {
    fn shift_cutoff(&self, cutoff: usize, by: (isize, isize)) -> Self {
        let Eqn { lhs, rhs } = self;

        Eqn { lhs: lhs.shift_cutoff(cutoff, by), rhs: rhs.shift_cutoff(cutoff, by) }
    }
}

impl ShiftCutoff for Exp {
    fn shift_cutoff(&self, cutoff: usize, by: (isize, isize)) -> Self {
        match self {
            Exp::Var { idx } => Exp::Var { idx: idx.shift_cutoff(cutoff, by) },
            Exp::TypCtor { name, args: subst } => {
                Exp::TypCtor { name: name.clone(), args: subst.shift_cutoff(cutoff, by) }
            }
            Exp::Ctor { name, args: subst } => {
                Exp::Ctor { name: name.clone(), args: subst.shift_cutoff(cutoff, by) }
            }
            Exp::Dtor { exp, name, args: subst } => Exp::Dtor {
                exp: exp.shift_cutoff(cutoff, by),
                name: name.clone(),
                args: subst.shift_cutoff(cutoff, by),
            },
            Exp::Anno { exp, typ } => {
                Exp::Anno { exp: exp.shift_cutoff(cutoff, by), typ: typ.shift_cutoff(cutoff, by) }
            }
            Exp::Type => Exp::Type,
        }
    }
}

impl AlphaEq for Exp {
    fn alpha_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Exp::Var { idx }, Exp::Var { idx: idx2 }) => idx.alpha_eq(idx2),
            (Exp::TypCtor { name, args: subst }, Exp::TypCtor { name: name2, args: subst2 }) => {
                name == name2 && subst.alpha_eq(subst2)
            }
            (Exp::Ctor { name, args: subst }, Exp::Ctor { name: name2, args: subst2 }) => {
                name == name2 && subst.alpha_eq(subst2)
            }
            (
                Exp::Dtor { exp, name, args: subst },
                Exp::Dtor { exp: exp2, name: name2, args: subst2 },
            ) => exp.alpha_eq(exp2) && name == name2 && subst.alpha_eq(subst2),
            (Exp::Anno { exp, typ }, Exp::Anno { exp: exp2, typ: typ2 }) => {
                exp.alpha_eq(exp2) && typ.alpha_eq(typ2)
            }
            (Exp::Type, Exp::Type) => true,
            (_, _) => false,
        }
    }
}

impl AlphaEq for Args {
    fn alpha_eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }

        self.iter().zip(other.iter()).all(|(lhs, rhs)| lhs.alpha_eq(rhs))
    }
}
