use std::rc::Rc;

use codespan::Span;
use data::HashMap;

use super::ast;
use super::common::*;
use super::de_bruijn::*;

#[derive(Debug, Clone)]
pub struct Prg {
    pub decls: Decls,
    pub exp: Option<Rc<Exp>>,
}

#[derive(Debug, Clone)]
pub struct Decls {
    /// Map from identifiers to declarations
    pub map: HashMap<Ident, Decl>,
    /// Order in which declarations are defined in the source
    pub order: Vec<Ident>,
}

impl Decls {
    pub fn empty() -> Self {
        Self { map: data::HashMap::default(), order: Vec::new() }
    }
}

#[derive(Debug, Clone)]
pub enum Decl {
    Data(Data),
    Codata(Codata),
    Ctor(Ctor),
    Dtor(Dtor),
    Def(Def),
    Codef(Codef),
}

#[derive(Debug, Clone)]
pub struct Data {
    pub info: Info,
    pub name: Ident,
    pub typ: TypAbs,
    pub ctors: Vec<Ident>,
    pub impl_block: Option<Impl>,
}

#[derive(Debug, Clone)]
pub struct Codata {
    pub info: Info,
    pub name: Ident,
    pub typ: TypAbs,
    pub dtors: Vec<Ident>,
    pub impl_block: Option<Impl>,
}

#[derive(Debug, Clone)]
pub struct Impl {
    pub info: Info,
    pub name: Ident,
    pub defs: Vec<Ident>,
}

impl From<ast::Impl> for Impl {
    fn from(ast: ast::Impl) -> Self {
        let ast::Impl { info, name, defs } = ast;

        Self { info: info.into(), name, defs }
    }
}

#[derive(Debug, Clone)]
pub struct TypAbs {
    pub params: Telescope,
}

#[derive(Debug, Clone)]
pub struct Ctor {
    pub info: Info,
    pub name: Ident,
    pub params: Telescope,
    pub typ: TypApp,
}

#[derive(Debug, Clone)]
pub struct Dtor {
    pub info: Info,
    pub name: Ident,
    pub params: Telescope,
    pub on_typ: TypApp,
    pub in_typ: Rc<Exp>,
}

#[derive(Debug, Clone)]
pub struct Def {
    pub info: Info,
    pub name: Ident,
    pub params: Telescope,
    pub on_typ: TypApp,
    pub in_typ: Rc<Exp>,
    pub body: Match,
}

impl Def {
    pub fn to_dtor(&self) -> Dtor {
        Dtor {
            info: self.info.clone(),
            name: self.name.clone(),
            params: self.params.clone(),
            on_typ: self.on_typ.clone(),
            in_typ: self.in_typ.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Codef {
    pub info: Info,
    pub name: Ident,
    pub params: Telescope,
    pub typ: TypApp,
    pub body: Comatch,
}

impl Codef {
    pub fn to_ctor(&self) -> Ctor {
        Ctor {
            info: self.info.clone(),
            name: self.name.clone(),
            params: self.params.clone(),
            typ: self.typ.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Match {
    pub info: Info,
    pub cases: Vec<Case>,
}

#[derive(Debug, Clone)]
pub struct Comatch {
    pub info: Info,
    // TODO: Consider renaming this field to "cocases"
    pub cases: Vec<Cocase>,
}

#[derive(Debug, Clone)]
pub struct Case {
    pub info: Info,
    pub name: Ident,
    pub args: Telescope,
    pub eqns: Eqns,
    /// Body being `None` represents an absurd pattern
    pub body: Option<Rc<Exp>>,
}

#[derive(Debug, Clone)]
pub struct Cocase {
    pub info: Info,
    pub name: Ident,
    pub args: Telescope,
    pub eqns: Eqns,
    /// Body being `None` represents an absurd pattern
    pub body: Option<Rc<Exp>>,
}

#[derive(Debug, Clone)]
pub struct Eqn {
    pub info: Info,
    pub lhs: Rc<Exp>,
    pub rhs: Rc<Exp>,
}

#[derive(Debug, Clone)]
pub struct TypApp {
    pub info: Info,
    pub name: Ident,
    pub args: Args,
}

#[derive(Debug, Clone)]
pub enum Exp {
    Var { info: TypedInfo, name: Ident, idx: Idx },
    TyCtor { info: TypedInfo, name: Ident, args: Args },
    Ctor { info: TypedInfo, name: Ident, args: Args },
    Dtor { info: TypedInfo, exp: Rc<Exp>, name: Ident, args: Args },
    Anno { info: TypedInfo, exp: Rc<Exp>, typ: Rc<Exp> },
    Type { info: TypedInfo },
}

/// Wrapper type signifying the wrapped parameters have telescope
/// semantics. I.e. each parameter binding in the parameter list is in scope
/// for the following parameters.
#[derive(Debug, Clone)]
pub struct Telescope(pub Params);

pub type Params = Vec<Param>;
pub type Args = Vec<Rc<Exp>>;

#[derive(Debug, Clone)]
pub struct Param {
    pub name: Ident,
    pub typ: Rc<Exp>,
}

#[derive(Debug, Clone)]
pub struct Eqns {
    pub eqns: Vec<Eqn>,
    pub params: Vec<EqnParam>,
}

impl Eqns {
    pub fn empty() -> Self {
        Self { eqns: vec![], params: vec![] }
    }
}

#[derive(Debug, Clone)]
pub struct EqnParam {
    pub name: Ident,
    pub eqn: Eqn,
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
            Exp::TyCtor { info, .. } => info,
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
