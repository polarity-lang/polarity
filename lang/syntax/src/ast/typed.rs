use std::rc::Rc;

use codespan::Span;

use crate::common::{HasInfo, HasSpan, Ident};
use crate::de_bruijn::Idx;

use super::generic;
use super::untyped;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct TST;

impl generic::Phase for TST {
    type Info = Info;
    type TypeInfo = TypeInfo;

    type VarName = Ident;

    fn print_var(name: &Self::VarName, _idx: Idx) -> String {
        name.clone()
    }
}

pub type Prg = generic::Prg<TST>;
pub type Decls = generic::Decls<TST>;
pub type Decl = generic::Decl<TST>;
pub type Data = generic::Data<TST>;
pub type Codata = generic::Codata<TST>;
pub type Impl = generic::Impl<TST>;
pub type TypAbs = generic::TypAbs<TST>;
pub type Ctor = generic::Ctor<TST>;
pub type Dtor = generic::Dtor<TST>;
pub type Def = generic::Def<TST>;
pub type Codef = generic::Codef<TST>;
pub type Match = generic::Match<TST>;
pub type Comatch = generic::Comatch<TST>;
pub type Case = generic::Case<TST>;
pub type Cocase = generic::Cocase<TST>;
pub type TypApp = generic::TypApp<TST>;
pub type Exp = generic::Exp<TST>;
pub type Telescope = generic::Telescope<TST>;
pub type Params = generic::Params<TST>;
pub type Args = generic::Args<TST>;
pub type Param = generic::Param<TST>;

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
