use codespan::Span;

use crate::common::*;
use crate::de_bruijn::*;
use crate::equiv::*;
use crate::tst;

use super::def::*;

impl<P: Phase> ShiftCutoff for Exp<P>
where
    P::Typ: ShiftCutoff,
{
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
            Exp::Match { info, name, on_exp, in_typ, body } => Exp::Match {
                info: info.clone(),
                name: name.clone(),
                on_exp: on_exp.shift_cutoff(cutoff, by),
                in_typ: in_typ.shift_cutoff(cutoff, by),
                body: body.shift_cutoff(cutoff, by),
            },
            Exp::Comatch { info, name, body } => Exp::Comatch {
                info: info.clone(),
                name: name.clone(),
                body: body.shift_cutoff(cutoff, by),
            },
            Exp::Hole {} => todo!(),
        }
    }
}

impl<P: Phase> ShiftCutoff for Match<P>
where
    P::Typ: ShiftCutoff,
{
    fn shift_cutoff(&self, cutoff: usize, by: (isize, isize)) -> Self {
        let Match { info, cases } = self;
        Match { info: info.clone(), cases: cases.shift_cutoff(cutoff, by) }
    }
}

impl<P: Phase> ShiftCutoff for Comatch<P>
where
    P::Typ: ShiftCutoff,
{
    fn shift_cutoff(&self, cutoff: usize, by: (isize, isize)) -> Self {
        let Comatch { info, cases } = self;
        Comatch { info: info.clone(), cases: cases.shift_cutoff(cutoff, by) }
    }
}

impl<P: Phase> ShiftCutoff for Case<P>
where
    P::Typ: ShiftCutoff,
{
    fn shift_cutoff(&self, cutoff: usize, by: (isize, isize)) -> Self {
        let Case { info, name, args, body } = self;
        Case {
            info: info.clone(),
            name: name.clone(),
            args: args.clone(),
            body: body.shift_cutoff(cutoff + 1, by),
        }
    }
}

impl<P: Phase> ShiftCutoff for Cocase<P>
where
    P::Typ: ShiftCutoff,
{
    fn shift_cutoff(&self, cutoff: usize, by: (isize, isize)) -> Self {
        let Cocase { info, name, args, body } = self;
        Cocase {
            info: info.clone(),
            name: name.clone(),
            args: args.clone(),
            body: body.shift_cutoff(cutoff + 1, by),
        }
    }
}

impl<P: Phase> ShiftCutoff for TypApp<P>
where
    P::Typ: ShiftCutoff,
{
    fn shift_cutoff(&self, cutoff: usize, by: (isize, isize)) -> Self {
        let TypApp { info, name, args } = self;
        TypApp { info: info.clone(), name: name.clone(), args: args.shift_cutoff(cutoff, by) }
    }
}

impl<P: Phase> AlphaEq for Exp<P> {
    fn alpha_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl<P: Phase> HasInfo for Decl<P> {
    type Info = P::Info;

    fn info(&self) -> Self::Info {
        match self {
            Decl::Data(data) => data.info.clone(),
            Decl::Codata(codata) => codata.info.clone(),
            Decl::Ctor(ctor) => ctor.info.clone(),
            Decl::Dtor(dtor) => dtor.info.clone(),
            Decl::Def(def) => def.info.clone(),
            Decl::Codef(codef) => codef.info.clone(),
        }
    }
}

impl<P: Phase> HasInfo for Exp<P> {
    type Info = P::TypeInfo;

    fn info(&self) -> Self::Info {
        match self {
            Exp::Var { info, .. } => info.clone(),
            Exp::TypCtor { info, .. } => info.clone(),
            Exp::Ctor { info, .. } => info.clone(),
            Exp::Dtor { info, .. } => info.clone(),
            Exp::Anno { info, .. } => info.clone(),
            Exp::Type { info } => info.clone(),
            Exp::Match { info, .. } => info.clone().into(),
            Exp::Comatch { info, .. } => info.clone().into(),
            Exp::Hole {} => todo!(),
        }
    }
}

impl<P: Phase> HasSpan for Exp<P> {
    fn span(&self) -> Option<Span> {
        self.info().span()
    }
}

impl ShiftCutoff for () {
    fn shift_cutoff(&self, _cutoff: usize, _by: (isize, isize)) -> Self {}
}

impl ShiftCutoff for tst::Typ {
    fn shift_cutoff(&self, cutoff: usize, by: (isize, isize)) -> Self {
        Self::from(self.as_exp().shift_cutoff(cutoff, by))
    }
}
