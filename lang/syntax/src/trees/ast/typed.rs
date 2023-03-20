use std::rc::Rc;

use codespan::Span;

use crate::common::*;
use crate::nf;

use super::generic;
use super::untyped;

#[derive(Default, Clone, Debug, Eq, PartialEq, Hash)]
pub struct TST;

impl generic::Phase for TST {
    type Info = Info;
    type TypeInfo = TypeInfo;
    type TypeAppInfo = TypeAppInfo;

    type VarName = Ident;
    type Label = Ident;
    type InfTyp = Typ;

    fn print_var(name: &Self::VarName, idx: Option<Idx>) -> String {
        if let Some(idx) = idx {
            format!("{name}@{idx}")
        } else {
            name.clone()
        }
    }

    fn print_label(name: &Self::Label) -> Option<String> {
        Some(name.to_owned())
    }
}

pub type Prg = generic::Prg<TST>;
pub type Decls = generic::Decls<TST>;
pub type Decl = generic::Decl<TST>;
pub type Type<'a> = generic::Type<'a, TST>;
pub type Data = generic::Data<TST>;
pub type Codata = generic::Codata<TST>;
pub type TypAbs = generic::TypAbs<TST>;
pub type Ctor = generic::Ctor<TST>;
pub type Dtor = generic::Dtor<TST>;
pub type Def = generic::Def<TST>;
pub type Codef = generic::Codef<TST>;
pub type Match = generic::Match<TST>;
pub type Comatch = generic::Comatch<TST>;
pub type Case = generic::Case<TST>;
pub type Cocase = generic::Cocase<TST>;
pub type SelfParam = generic::SelfParam<TST>;
pub type TypApp = generic::TypApp<TST>;
pub type Exp = generic::Exp<TST>;
pub type Motive = generic::Motive<TST>;
pub type Telescope = generic::Telescope<TST>;
pub type TelescopeInst = generic::TelescopeInst<TST>;
pub type Args = generic::Args<TST>;
pub type Param = generic::Param<TST>;
pub type ParamInst = generic::ParamInst<TST>;

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
    pub typ: Rc<nf::Nf>,
    pub span: Option<Span>,
}

impl From<Rc<nf::Nf>> for TypeInfo {
    fn from(typ: Rc<nf::Nf>) -> Self {
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
    pub typ_nf: nf::TypApp,
    pub span: Option<Span>,
}

impl From<TypeAppInfo> for TypeInfo {
    fn from(type_app_info: TypeAppInfo) -> Self {
        let nf::TypApp { info, name, args } = type_app_info.typ_nf;
        Self { span: info.span, typ: Rc::new(nf::Nf::TypCtor { info, name, args }) }
    }
}

impl HasSpan for TypeAppInfo {
    fn span(&self) -> Option<Span> {
        self.span
    }
}

pub trait HasTypeInfo {
    fn typ(&self) -> Rc<nf::Nf>;
}

impl<T: HasInfo<Info = TypeInfo>> HasTypeInfo for T {
    fn typ(&self) -> Rc<nf::Nf> {
        self.info().typ
    }
}

impl From<untyped::Info> for Info {
    fn from(info: untyped::Info) -> Self {
        Self { span: info.span }
    }
}

pub trait ElabInfoExt {
    fn with_type(&self, typ: Rc<nf::Nf>) -> TypeInfo;
    fn with_type_app(&self, typ: TypApp, typ_nf: nf::TypApp) -> TypeAppInfo;
}

impl ElabInfoExt for untyped::Info {
    fn with_type(&self, typ: Rc<nf::Nf>) -> TypeInfo {
        TypeInfo { typ, span: self.span }
    }

    fn with_type_app(&self, typ: TypApp, typ_nf: nf::TypApp) -> TypeAppInfo {
        TypeAppInfo { typ, typ_nf, span: self.span }
    }
}
