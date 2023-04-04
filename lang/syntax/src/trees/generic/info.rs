use codespan::Span;

use crate::common::*;

use super::def::*;

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
            Exp::Hole { info, .. } => info.clone(),
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
