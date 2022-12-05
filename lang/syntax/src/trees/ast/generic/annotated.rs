use std::rc::Rc;

use crate::ast::*;
use crate::tst;
use crate::wst;

pub trait Annotated<P: Phase> {
    fn typ(&self) -> Rc<Exp<P>>;
}

impl<P: Phase> Annotated<P> for Rc<Exp<P>> {
    fn typ(&self) -> Rc<Exp<P>> {
        self.clone()
    }
}

impl<P: Phase, T1, T2: Annotated<P>> Annotated<P> for (T1, T2) {
    fn typ(&self) -> Rc<Exp<P>> {
        self.1.typ()
    }
}

impl<P: Phase> Annotated<P> for Param<P> {
    fn typ(&self) -> Rc<Exp<P>> {
        self.typ.clone()
    }
}

impl<P: Phase> Annotated<P> for &Param<P> {
    fn typ(&self) -> Rc<Exp<P>> {
        self.typ.clone()
    }
}

impl Annotated<tst::TST> for tst::ParamInst {
    fn typ(&self) -> Rc<tst::Exp> {
        self.typ.as_exp().clone()
    }
}

impl Annotated<tst::TST> for &tst::ParamInst {
    fn typ(&self) -> Rc<tst::Exp> {
        self.typ.as_exp().clone()
    }
}

impl Annotated<wst::WST> for wst::ParamInst {
    fn typ(&self) -> Rc<wst::Exp> {
        self.typ.as_exp().clone()
    }
}

impl Annotated<wst::WST> for &wst::ParamInst {
    fn typ(&self) -> Rc<wst::Exp> {
        self.typ.as_exp().clone()
    }
}
