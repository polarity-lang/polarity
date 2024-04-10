//! AST with type information

use std::rc::Rc;

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
    pub ctx: Option<TypeCtx>,
}

impl From<Rc<ust::Exp>> for TypeInfo {
    fn from(typ: Rc<ust::Exp>) -> Self {
        TypeInfo { typ, ctx: None }
    }
}

#[derive(Debug, Clone)]
pub struct TypeAppInfo {
    pub typ: TypApp,
    pub typ_nf: ust::TypApp,
}

impl From<TypeAppInfo> for TypeInfo {
    fn from(type_app_info: TypeAppInfo) -> Self {
        let ust::TypApp { info, name, args, span } = type_app_info.typ_nf;
        Self { typ: Rc::new(ust::Exp::TypCtor { info, span, name, args }), ctx: None }
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
            Exp::Type { info, .. } => info.clone().typ,
            Exp::Match { info, .. } => {
                let ust::TypApp { info, span, name, args } = info.clone().typ_nf;
                Rc::new(ust::Exp::TypCtor { info, span, name, args })
            }
            Exp::Comatch { info, .. } => {
                let ust::TypApp { info, span, name, args } = info.clone().typ_nf;
                Rc::new(ust::Exp::TypCtor { info, span, name, args })
            }
            Exp::Hole { info, .. } => info.clone().typ,
        }
    }
}
