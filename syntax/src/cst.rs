use std::rc::Rc;

use super::common::*;

#[derive(Debug, Clone)]
pub struct Prg {
    pub decls: Vec<Decl>,
    pub exp: Option<Rc<Exp>>,
}

#[derive(Debug, Clone)]
pub enum Decl {
    Data(Data),
    Codata(Codata),
    Cns(Cns),
    Prd(Prd),
}

#[derive(Debug, Clone)]
pub struct Data {
    pub name: Ident,
    pub params: Params,
    pub ctors: Vec<Ctor>,
}

#[derive(Debug, Clone)]
pub struct Codata {
    pub name: Ident,
    pub params: Params,
    pub ctors: Vec<Dtor>,
}

#[derive(Debug, Clone)]
pub struct Ctor {
    pub name: Ident,
    pub params: Params,
    pub typ: TypApp,
}

#[derive(Debug, Clone)]
pub struct Dtor {
    pub name: Ident,
    pub params: Params,
    pub on_typ: TypApp,
    pub in_typ: Rc<Exp>,
}

#[derive(Debug, Clone)]
pub struct Cns {
    pub name: Ident,
    pub params: Params,
    pub on_typ: TypApp,
    pub in_typ: Rc<Exp>,
    pub body: Match,
}

#[derive(Debug, Clone)]
pub struct Prd {
    pub name: Ident,
    pub params: Params,
    pub typ: TypApp,
    pub body: Comatch,
}

#[derive(Debug, Clone)]
pub struct Match {
    pub cases: Vec<Case>,
}

#[derive(Debug, Clone)]
pub struct Comatch {
    pub cases: Vec<Case>,
}

#[derive(Debug, Clone)]
pub struct Case {
    pub name: Ident,
    pub args: Params,
    pub body: Rc<Exp>,
}

#[derive(Debug, Clone)]
pub struct TypApp {
    pub name: Ident,
    pub subst: Subst,
}

#[derive(Debug, Clone)]
pub enum Exp {
    Var { name: Ident },
    Ctor { name: Ident, subst: Subst },
    Dtor { exp: Rc<Exp>, name: Ident, subst: Subst },
    Ano { exp: Rc<Exp>, typ: Rc<Exp> },
    Type,
}

pub type Params = Vec<Param>;
pub type Subst = Vec<Rc<Exp>>;

#[derive(Debug, Clone)]
pub struct Param {
    pub name: Ident,
    pub typ: Rc<Exp>,
}
