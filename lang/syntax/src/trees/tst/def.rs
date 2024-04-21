//! AST with type information

use std::rc::Rc;

use codespan::Span;

use crate::common::*;
use crate::ctx::values::TypeCtx;
use crate::ust;

use crate::generic;

#[derive(Default, Clone, Debug, Eq, PartialEq, Hash)]
pub struct TST;

impl generic::Phase for TST {
    type TypeInfo = TypeInfo;
    type TypeAppInfo = TypeAppInfo;

    type InfTyp = Typ;
    type Ctx = TypeCtx;
}

pub type Ident = generic::Ident;
pub type Label = generic::Label;
pub type DocComment = generic::DocComment;
pub type Attribute = generic::Attribute;
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
pub type Let = generic::Let<TST>;
pub type Match = generic::Match<TST>;
pub type Case = generic::Case<TST>;
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

#[derive(Debug, Clone)]
pub struct TypeInfo {
    pub typ: Rc<ust::Exp>,
    pub span: Option<Span>,
    pub ctx: Option<TypeCtx>,
}

impl From<Rc<ust::Exp>> for TypeInfo {
    fn from(typ: Rc<ust::Exp>) -> Self {
        TypeInfo { span: typ.span(), typ, ctx: None }
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
    pub typ_nf: ust::TypApp,
    pub span: Option<Span>,
}

impl From<TypeAppInfo> for TypeInfo {
    fn from(type_app_info: TypeAppInfo) -> Self {
        let ust::TypApp { info, name, args } = type_app_info.typ_nf;
        Self { span: info.span(), typ: Rc::new(ust::Exp::TypCtor { info, name, args }), ctx: None }
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
impl HasTypeInfo for Exp {
    fn typ(&self) -> Rc<ust::Exp> {
        match self {
            Exp::Var { info, .. } => info.clone().typ,
            Exp::TypCtor { info, .. } => info.clone().typ,
            Exp::Ctor { info, .. } => info.clone().typ,
            Exp::Dtor { info, .. } => info.clone().typ,
            Exp::Anno { info, .. } => info.clone().typ,
            Exp::Type { info } => info.clone().typ,
            Exp::LocalMatch { info, .. } => {
                let ust::TypApp { info, name, args } = info.clone().typ_nf;
                Rc::new(ust::Exp::TypCtor { info, name, args })
            }
            Exp::LocalComatch { info, .. } => {
                let ust::TypApp { info, name, args } = info.clone().typ_nf;
                Rc::new(ust::Exp::TypCtor { info, name, args })
            }
            Exp::Hole { info, .. } => info.clone().typ,
        }
    }
}

pub trait ElabInfoExt {
    fn with_type(&self, typ: Rc<ust::Exp>) -> TypeInfo;
    fn with_type_and_ctx(&self, typ: Rc<ust::Exp>, ctx: TypeCtx) -> TypeInfo;
    fn with_type_app(&self, typ: TypApp, typ_nf: ust::TypApp) -> TypeAppInfo;
}

impl ElabInfoExt for Option<Span> {
    fn with_type(&self, typ: Rc<ust::Exp>) -> TypeInfo {
        TypeInfo { typ, span: *self, ctx: None }
    }

    fn with_type_and_ctx(&self, typ: Rc<ust::Exp>, ctx: TypeCtx) -> TypeInfo {
        TypeInfo { typ, span: *self, ctx: Some(ctx) }
    }

    fn with_type_app(&self, typ: TypApp, typ_nf: ust::TypApp) -> TypeAppInfo {
        TypeAppInfo { typ, typ_nf, span: *self }
    }
}
