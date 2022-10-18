use codespan::Span;

use crate::common::*;
use crate::de_bruijn::*;
use crate::equiv::*;

use super::def::*;

impl ShiftCutoff for Exp {
    fn shift_cutoff(&self, cutoff: usize, by: (isize, isize)) -> Self {
        match self {
            Exp::Var { info, name, idx } => Exp::Var {
                info: info.clone(),
                name: name.clone(),
                idx: idx.shift_cutoff(cutoff, by),
            },
            Exp::TypCtor { info, name, args: subst } => Exp::TypCtor {
                info: info.clone(),
                name: name.clone(),
                args: subst.shift_cutoff(cutoff, by),
            },
            Exp::Ctor { info, name, args: subst } => Exp::Ctor {
                info: info.clone(),
                name: name.clone(),
                args: subst.shift_cutoff(cutoff, by),
            },
            Exp::Dtor { info, exp, name, args: subst } => Exp::Dtor {
                info: info.clone(),
                exp: exp.shift_cutoff(cutoff, by),
                name: name.clone(),
                args: subst.shift_cutoff(cutoff, by),
            },
            Exp::Anno { info, exp, typ } => Exp::Anno {
                info: info.clone(),
                exp: exp.shift_cutoff(cutoff, by),
                typ: typ.shift_cutoff(cutoff, by),
            },
            Exp::Type { info } => Exp::Type { info: info.clone() },
        }
    }
}

impl AlphaEq for Exp {
    fn alpha_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Exp::Var { info: _, name: _, idx }, Exp::Var { info: _, name: _, idx: idx2 }) => {
                idx.alpha_eq(idx2)
            }
            (
                Exp::TypCtor { info: _, name, args: subst },
                Exp::TypCtor { info: _, name: name2, args: subst2 },
            ) => name == name2 && subst.alpha_eq(subst2),
            (
                Exp::Ctor { info: _, name, args: subst },
                Exp::Ctor { info: _, name: name2, args: subst2 },
            ) => name == name2 && subst.alpha_eq(subst2),
            (
                Exp::Dtor { info: _, exp, name, args: subst },
                Exp::Dtor { info: _, exp: exp2, name: name2, args: subst2 },
            ) => exp.alpha_eq(exp2) && name == name2 && subst.alpha_eq(subst2),
            (Exp::Anno { info: _, exp, typ }, Exp::Anno { info: _, exp: exp2, typ: typ2 }) => {
                exp.alpha_eq(exp2) && typ.alpha_eq(typ2)
            }
            (Exp::Type { info: _ }, Exp::Type { info: _ }) => true,
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

impl HasInfo for Exp {
    type Info = Info;

    fn info(&self) -> &Self::Info {
        match self {
            Exp::Var { info, .. } => info,
            Exp::TypCtor { info, .. } => info,
            Exp::Ctor { info, .. } => info,
            Exp::Dtor { info, .. } => info,
            Exp::Anno { info, .. } => info,
            Exp::Type { info } => info,
        }
    }

    fn span(&self) -> Option<&Span> {
        self.info().span.as_ref()
    }
}
