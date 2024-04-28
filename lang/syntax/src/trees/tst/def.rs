//! AST with type information

use std::rc::Rc;

use crate::ctx::values::TypeCtx;
use crate::generic::TypeUniv;
use crate::ust;

use crate::generic;

#[derive(Default, Clone, Debug, Eq, PartialEq, Hash)]
pub struct TST;

impl generic::Phase for TST {
    type TypeInfo = TypeInfo;
    type TypeAppInfo = TypeAppInfo;
}

pub type Ident = generic::Ident;
pub type Label = generic::Label;
pub type DocComment = generic::DocComment;
pub type Attribute = generic::Attribute;
pub type Prg = generic::Prg<TST>;
pub type Decls = generic::Decls<TST>;
pub type Decl = generic::Decl<TST>;
pub type DataCodata<'a> = generic::DataCodata<'a, TST>;
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
pub type Exp = generic::Exp<TST>;
pub type Motive = generic::Motive<TST>;
pub type Telescope = generic::Telescope<TST>;
pub type TelescopeInst = generic::TelescopeInst<TST>;
pub type Args = generic::Args<TST>;
pub type Param = generic::Param<TST>;
pub type ParamInst = generic::ParamInst<TST>;

pub type TypCtor = generic::TypCtor<TST>;
pub type Call = generic::Call<TST>;
pub type DotCall = generic::DotCall<TST>;
pub type Anno = generic::Anno<TST>;
pub type LocalMatch = generic::LocalMatch<TST>;
pub type LocalComatch = generic::LocalComatch<TST>;

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
    pub typ: TypCtor,
    pub typ_nf: ust::TypCtor,
}

pub trait HasTypeInfo {
    fn typ(&self) -> Option<Rc<ust::Exp>>;
}

impl HasTypeInfo for Exp {
    fn typ(&self) -> Option<Rc<ust::Exp>> {
        match self {
            Exp::Variable(e) => e.inferred_type.clone(),
            Exp::TypCtor(_) => Some(Rc::new(ust::Exp::TypeUniv(TypeUniv { span: None }))),
            Exp::Call(e) => Some(e.info.clone().typ),
            Exp::DotCall(e) => Some(e.info.clone().typ),
            Exp::Anno(e) => e.normalized_type.clone(),
            Exp::TypeUniv(_) => Some(Rc::new(ust::Exp::TypeUniv(TypeUniv { span: None }))),
            Exp::LocalMatch(e) => {
                let ust::TypCtor { span, name, args } = e.info.clone().typ_nf;
                Some(Rc::new(ust::Exp::TypCtor(ust::TypCtor { span, name, args })))
            }
            Exp::LocalComatch(e) => {
                let ust::TypCtor { span, name, args } = e.info.clone().typ_nf;
                Some(Rc::new(ust::Exp::TypCtor(ust::TypCtor { span, name, args })))
            }
            Exp::Hole(e) => e.inferred_type.clone(),
        }
    }
}
