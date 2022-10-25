use codespan::Span;

use crate::common::*;
use crate::de_bruijn::*;
use crate::equiv::*;

use super::def::*;

impl<P: Phase> ShiftCutoff for Exp<P> {
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
            Exp::Match { .. } => unimplemented!(), // TODO: Implement
            Exp::Comatch { .. } => unimplemented!(), // TODO: Implement
        }
    }
}

impl<P: Phase> AlphaEq for Exp<P> {
    fn alpha_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl<P: Phase> HasInfo for Exp<P> {
    type Info = P::TypeInfo;

    fn info(&self) -> &Self::Info {
        match self {
            Exp::Var { info, .. } => info,
            Exp::TypCtor { info, .. } => info,
            Exp::Ctor { info, .. } => info,
            Exp::Dtor { info, .. } => info,
            Exp::Anno { info, .. } => info,
            Exp::Type { info } => info,
            Exp::Match { .. } => unimplemented!(), // TODO: Implement
            Exp::Comatch { .. } => unimplemented!(), // TODO: Implement
        }
    }
}

impl<P: Phase> HasSpan for Exp<P> {
    fn span(&self) -> Option<&Span> {
        self.info().span()
    }
}
