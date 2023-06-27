use codespan::Span;

use crate::common::*;

use super::def::*;

impl<P: Phase> HasSpan for Decl<P> {
    fn span(&self) -> Option<Span> {
        match self {
            Decl::Data(data) => data.info,
            Decl::Codata(codata) => codata.info,
            Decl::Ctor(ctor) => ctor.info,
            Decl::Dtor(dtor) => dtor.info,
            Decl::Def(def) => def.info,
            Decl::Codef(codef) => codef.info,
        }
    }
}

impl<P: Phase> HasSpan for Exp<P> {
    fn span(&self) -> Option<Span> {
        match self {
            Exp::Var { info, .. } => info.span(),
            Exp::TypCtor { info, .. } => info.span(),
            Exp::Ctor { info, .. } => info.span(),
            Exp::Dtor { info, .. } => info.span(),
            Exp::Anno { info, .. } => info.span(),
            Exp::Type { info } => info.span(),
            Exp::Match { info, .. } => info.span(),
            Exp::Comatch { info, .. } => info.span(),
            Exp::Hole { info, .. } => info.span(),
        }
    }
}

impl ShiftInRange for () {
    fn shift_in_range<R: ShiftRange>(&self, _range: R, _by: (isize, isize)) -> Self {}
}
