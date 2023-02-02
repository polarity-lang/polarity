use codespan::Span;

use crate::common::*;
use crate::tst;
use crate::wst;

use super::def::*;

impl<P: Phase> ShiftInRange for Exp<P>
where
    P::InfTyp: ShiftInRange,
{
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        match self {
            Exp::Var { info, name, idx } => Exp::Var {
                info: info.clone(),
                name: name.clone(),
                idx: idx.shift_in_range(range, by),
            },
            Exp::TypCtor { info, name, args: subst } => Exp::TypCtor {
                info: info.clone(),
                name: name.clone(),
                args: subst.shift_in_range(range, by),
            },
            Exp::Ctor { info, name, args: subst } => Exp::Ctor {
                info: info.clone(),
                name: name.clone(),
                args: subst.shift_in_range(range, by),
            },
            Exp::Dtor { info, exp, name, args: subst } => Exp::Dtor {
                info: info.clone(),
                exp: exp.shift_in_range(range.clone(), by),
                name: name.clone(),
                args: subst.shift_in_range(range, by),
            },
            Exp::Anno { info, exp, typ } => Exp::Anno {
                info: info.clone(),
                exp: exp.shift_in_range(range.clone(), by),
                typ: typ.shift_in_range(range, by),
            },
            Exp::Type { info } => Exp::Type { info: info.clone() },
            Exp::Match { info, name, on_exp, motive, ret_typ, body } => Exp::Match {
                info: info.clone(),
                name: name.clone(),
                on_exp: on_exp.shift_in_range(range.clone(), by),
                motive: motive.shift_in_range(range.clone(), by),
                ret_typ: ret_typ.shift_in_range(range.clone(), by),
                body: body.shift_in_range(range, by),
            },
            Exp::Comatch { info, name, body } => Exp::Comatch {
                info: info.clone(),
                name: name.clone(),
                body: body.shift_in_range(range, by),
            },
            Exp::Hole { info } => Exp::Hole { info: info.clone() },
        }
    }
}

impl<P: Phase> ShiftInRange for Motive<P>
where
    P::InfTyp: ShiftInRange,
{
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let Motive { info, param, ret_typ } = self;

        Motive {
            info: info.clone(),
            param: param.clone(),
            ret_typ: ret_typ.shift_in_range(range.shift(1), by),
        }
    }
}

impl<P: Phase> ShiftInRange for Match<P>
where
    P::InfTyp: ShiftInRange,
{
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let Match { info, cases } = self;
        Match { info: info.clone(), cases: cases.shift_in_range(range, by) }
    }
}

impl<P: Phase> ShiftInRange for Comatch<P>
where
    P::InfTyp: ShiftInRange,
{
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let Comatch { info, cases } = self;
        Comatch { info: info.clone(), cases: cases.shift_in_range(range, by) }
    }
}

impl<P: Phase> ShiftInRange for Case<P>
where
    P::InfTyp: ShiftInRange,
{
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let Case { info, name, args, body } = self;
        Case {
            info: info.clone(),
            name: name.clone(),
            args: args.clone(),
            body: body.shift_in_range(range.shift(1), by),
        }
    }
}

impl<P: Phase> ShiftInRange for Cocase<P>
where
    P::InfTyp: ShiftInRange,
{
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let Cocase { info, name, params: args, body } = self;
        Cocase {
            info: info.clone(),
            name: name.clone(),
            params: args.clone(),
            body: body.shift_in_range(range.shift(1), by),
        }
    }
}

impl<P: Phase> ShiftInRange for TypApp<P>
where
    P::InfTyp: ShiftInRange,
{
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let TypApp { info, name, args } = self;
        TypApp { info: info.clone(), name: name.clone(), args: args.shift_in_range(range, by) }
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
            Exp::Hole { info } => info.clone(),
        }
    }
}

impl<P: Phase> HasSpan for Exp<P> {
    fn span(&self) -> Option<Span> {
        self.info().span()
    }
}

impl ShiftInRange for () {
    fn shift_in_range<R: ShiftRange>(&self, _range: R, _by: (isize, isize)) -> Self {}
}

impl ShiftInRange for tst::Typ {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        Self::from(self.as_exp().shift_in_range(range, by))
    }
}

impl ShiftInRange for wst::Typ {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        Self::from(self.as_exp().shift_in_range(range, by))
    }
}
