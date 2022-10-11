use std::rc::Rc;

use codespan::ByteIndex;
use codespan::Span;

use super::common::*;
use super::de_bruijn::*;

#[derive(Debug, Clone)]
pub struct Prg {
    pub items: Vec<Item>,
    pub exp: Option<Rc<Exp>>,
}

#[derive(Debug, Clone)]
pub enum Item {
    Type(TypDecl),
    Impl(Impl),
}

#[derive(Debug, Clone)]
pub enum TypDecl {
    Data(Data),
    Codata(Codata),
}

#[derive(Debug, Clone)]
pub struct Data {
    pub info: Info,
    pub name: Ident,
    pub params: Telescope,
    pub ctors: Vec<Ctor>,
}

#[derive(Debug, Clone)]
pub struct Codata {
    pub info: Info,
    pub name: Ident,
    pub params: Telescope,
    pub dtors: Vec<Dtor>,
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
pub struct Impl {
    pub info: Info,
    pub name: Ident,
    pub decls: Vec<DefDecl>,
}

#[derive(Debug, Clone)]
pub enum DefDecl {
    Def(Def),
    Codef(Codef),
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

#[derive(Debug, Clone)]
pub struct Codef {
    pub info: Info,
    pub name: Ident,
    pub params: Telescope,
    pub typ: TypApp,
    pub body: Comatch,
}

#[derive(Debug, Clone)]
pub struct Match {
    pub info: Info,
    pub cases: Vec<Case>,
}

#[derive(Debug, Clone)]
pub struct Comatch {
    pub info: Info,
    pub cases: Vec<Cocase>,
}

#[derive(Debug, Clone)]
pub struct Case {
    pub info: Info,
    pub name: Ident,
    pub args: Telescope,
    pub eqns: EqnParams,
    /// Body being `None` represents an absurd pattern
    pub body: Option<Rc<Exp>>,
}

#[derive(Debug, Clone)]
pub struct Cocase {
    pub info: Info,
    pub name: Ident,
    pub args: Telescope,
    pub eqns: EqnParams,
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
    Call { info: Info, name: Ident, args: Args },
    DotCall { info: Info, exp: Rc<Exp>, name: Ident, args: Args },
    Anno { info: Info, exp: Rc<Exp>, typ: Rc<Exp> },
    Type { info: Info },
}

/// Wrapper type signifying the wrapped parameters have telescope
/// semantics. I.e. each parameter binding in the parameter list is in scope
/// for the following parameters. This influences the lowering semantic.
#[derive(Debug, Clone)]
pub struct Telescope(pub Params);

pub type Params = Vec<Param>;
pub type EqnParams = Vec<EqnParam>;
pub type Args = Vec<Rc<Exp>>;

#[derive(Debug, Clone)]
pub struct Param {
    pub name: Ident,
    pub typ: Rc<Exp>,
}

#[derive(Debug, Clone)]
pub struct EqnParam {
    pub name: Ident,
    pub eqn: Eqn,
}

#[derive(Debug, Clone)]
pub enum Var {
    Bound(Idx),
    Free(Ident),
}

#[derive(Debug, Clone)]
pub struct Info {
    pub span: Span,
}

impl Info {
    pub fn spanned<I: Into<ByteIndex>>(l: I, r: I) -> Self {
        Self { span: Span::new(l, r) }
    }
}
