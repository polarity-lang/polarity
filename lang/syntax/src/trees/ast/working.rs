use std::rc::Rc;

use codespan::Span;

use crate::common::*;
use crate::ust;

use super::generic;

#[derive(Default, Clone, Debug, Eq, PartialEq, Hash)]
pub struct WST;

impl generic::Phase for WST {
    type Info = Info;
    type TypeInfo = TypeInfo;
    type TypeAppInfo = TypeAppInfo;

    type VarName = Ident;
    type Typ = Typ;

    fn print_var(name: &Self::VarName, idx: Option<Idx>) -> String {
        if let Some(idx) = idx {
            format!("{}@{}", name, idx)
        } else {
            name.clone()
        }
    }
}

pub type Prg = generic::Prg<WST>;
pub type Decls = generic::Decls<WST>;
pub type Decl = generic::Decl<WST>;
pub type Type<'a> = generic::Type<'a, WST>;
pub type Data = generic::Data<WST>;
pub type Codata = generic::Codata<WST>;
pub type Impl = generic::Impl<WST>;
pub type TypAbs = generic::TypAbs<WST>;
pub type Ctor = generic::Ctor<WST>;
pub type Dtor = generic::Dtor<WST>;
pub type Def = generic::Def<WST>;
pub type Codef = generic::Codef<WST>;
pub type Match = generic::Match<WST>;
pub type Comatch = generic::Comatch<WST>;
pub type Case = generic::Case<WST>;
pub type Cocase = generic::Cocase<WST>;
pub type SelfParam = generic::SelfParam<WST>;
pub type TypApp = generic::TypApp<WST>;
pub type Exp = generic::Exp<WST>;
pub type Telescope = generic::Telescope<WST>;
pub type TelescopeInst = generic::TelescopeInst<WST>;
pub type Params = generic::Params<WST>;
pub type Args = generic::Args<WST>;
pub type Param = generic::Param<WST>;
pub type ParamInst = generic::ParamInst<WST>;

#[derive(Clone, Debug)]
pub struct Typ(Rc<Exp>);

impl Typ {
    pub fn as_exp(&self) -> &Rc<Exp> {
        &self.0
    }
}

impl From<Rc<Exp>> for Typ {
    fn from(exp: Rc<Exp>) -> Self {
        Self(exp)
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
    fn span(&self) -> Option<Span> {
        self.span
    }
}

#[derive(Debug, Clone)]
pub struct TypeInfo {
    pub typ: Rc<ust::Exp>,
    pub span: Option<Span>,
}

impl From<Rc<ust::Exp>> for TypeInfo {
    fn from(typ: Rc<ust::Exp>) -> Self {
        TypeInfo { span: typ.span(), typ }
    }
}

impl HasSpan for TypeInfo {
    fn span(&self) -> Option<Span> {
        self.span
    }
}

#[derive(Debug, Clone)]
pub struct TypeAppInfo {
    pub typ: TypApp,
    pub span: Option<Span>,
}

impl From<TypeAppInfo> for TypeInfo {
    fn from(type_app_info: TypeAppInfo) -> Self {
        Self { span: type_app_info.span, typ: Rc::new(type_app_info.typ.forget().to_exp()) }
    }
}

impl HasSpan for TypeAppInfo {
    fn span(&self) -> Option<Span> {
        self.span
    }
}

pub trait HasTypeInfo {
    fn typ(&self) -> Rc<ust::Exp>;
}

impl<T: HasInfo<Info = TypeInfo>> HasTypeInfo for T {
    fn typ(&self) -> Rc<ust::Exp> {
        self.info().typ
    }
}

impl From<ust::Info> for Info {
    fn from(info: ust::Info) -> Self {
        Self { span: info.span }
    }
}
