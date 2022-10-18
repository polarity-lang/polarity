use std::rc::Rc;

use codespan::Span;

use crate::common::{HasInfo, HasSpan, Ident};
use crate::de_bruijn::Idx;

use super::generic;
use super::untyped;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Elab;

impl generic::Phase for Elab {
    type Info = Info;
    type TypeInfo = TypeInfo;

    type VarName = Ident;

    fn print_var(name: &Self::VarName, _idx: Idx) -> String {
        name.clone()
    }
}

pub type Prg = generic::Prg<Elab>;
pub type Decls = generic::Decls<Elab>;
pub type Decl = generic::Decl<Elab>;
pub type Data = generic::Data<Elab>;
pub type Codata = generic::Codata<Elab>;
pub type Impl = generic::Impl<Elab>;
pub type TypAbs = generic::TypAbs<Elab>;
pub type Ctor = generic::Ctor<Elab>;
pub type Dtor = generic::Dtor<Elab>;
pub type Def = generic::Def<Elab>;
pub type Codef = generic::Codef<Elab>;
pub type Match = generic::Match<Elab>;
pub type Comatch = generic::Comatch<Elab>;
pub type Case = generic::Case<Elab>;
pub type Cocase = generic::Cocase<Elab>;
pub type TypApp = generic::TypApp<Elab>;
pub type Exp = generic::Exp<Elab>;
pub type Telescope = generic::Telescope<Elab>;
pub type Params = generic::Params<Elab>;
pub type Args = generic::Args<Elab>;
pub type Param = generic::Param<Elab>;

impl From<untyped::Impl> for Impl {
    fn from(untyped::Impl { info, name, defs }: untyped::Impl) -> Self {
        Self { info: info.into(), name, defs }
    }
}

#[derive(Default, Debug, Clone)]
pub struct Info {
    pub span: Option<Span>,
}

impl Info {
    pub fn empty() -> Self {
        Self { span: None }
    }
}

impl HasSpan for Info {
    fn span(&self) -> Option<&Span> {
        self.span.as_ref()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TypeInfo {
    pub typ: Rc<untyped::Exp>,
    pub span: Option<Span>,
}

impl From<Rc<untyped::Exp>> for TypeInfo {
    fn from(typ: Rc<untyped::Exp>) -> Self {
        TypeInfo { span: typ.span().cloned(), typ }
    }
}

impl HasSpan for TypeInfo {
    fn span(&self) -> Option<&Span> {
        self.span.as_ref()
    }
}

pub trait HasType {
    fn typ(&self) -> &Rc<untyped::Exp>;
}

impl<T: HasInfo<Info = TypeInfo>> HasType for T {
    fn typ(&self) -> &Rc<untyped::Exp> {
        &self.info().typ
    }
}

impl From<untyped::Info> for Info {
    fn from(info: untyped::Info) -> Self {
        Self { span: info.span }
    }
}

pub trait ElabInfoExt {
    fn with_type(&self, typ: Rc<untyped::Exp>) -> TypeInfo;
}

impl ElabInfoExt for untyped::Info {
    fn with_type(&self, typ: Rc<untyped::Exp>) -> TypeInfo {
        TypeInfo { typ, span: self.span }
    }
}
