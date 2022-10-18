use std::rc::Rc;

use codespan::Span;

use crate::ast;
use crate::common::{HasInfo, Ident};
use crate::generic;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Elab;

impl generic::Phase for Elab {
    type Info = Info;
    type TypeInfo = TypedInfo;

    type VarName = Ident;
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

impl From<ast::Impl> for Impl {
    fn from(ast: ast::Impl) -> Self {
        let ast::Impl { info, name, defs } = ast;

        Self { info: info.into(), name, defs }
    }
}

#[derive(Debug, Clone)]
pub struct Info {
    pub span: Option<Span>,
}

impl Info {
    pub fn empty() -> Self {
        Self { span: None }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TypedInfo {
    pub typ: Rc<ast::Exp>,
    pub span: Option<Span>,
}

impl From<Rc<ast::Exp>> for TypedInfo {
    fn from(typ: Rc<ast::Exp>) -> Self {
        TypedInfo { span: typ.span().cloned(), typ }
    }
}

impl HasInfo for Exp {
    type Info = TypedInfo;

    fn info(&self) -> &Self::Info {
        match self {
            Exp::Var { info, .. } => info,
            Exp::TypCtor { info, .. } => info,
            Exp::Ctor { info, .. } => info,
            Exp::Dtor { info, .. } => info,
            Exp::Anno { info, .. } => info,
            Exp::Type { info } => info,
        }
    }

    fn span(&self) -> Option<&Span> {
        self.info().span.as_ref()
    }
}

pub trait HasType {
    fn typ(&self) -> &Rc<ast::Exp>;
}

impl<T: HasInfo<Info = TypedInfo>> HasType for T {
    fn typ(&self) -> &Rc<ast::Exp> {
        &self.info().typ
    }
}

impl From<ast::Info> for Info {
    fn from(info: ast::Info) -> Self {
        Self { span: info.span }
    }
}

pub trait ElabInfoExt {
    fn with_type(&self, typ: Rc<ast::Exp>) -> TypedInfo;
}

impl ElabInfoExt for ast::Info {
    fn with_type(&self, typ: Rc<ast::Exp>) -> TypedInfo {
        TypedInfo { typ, span: self.span }
    }
}
