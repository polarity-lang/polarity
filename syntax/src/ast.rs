use std::collections::HashMap;
use std::rc::Rc;

use super::common::*;
use super::var::*;

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
        Self { map: HashMap::new(), order: Vec::new() }
    }
}

#[derive(Debug, Clone)]
pub enum Decl {
    Data(Data),
    Codata(Codata),
    Def(Def),
    Codef(Codef),
}

impl Decl {
    pub fn name(&self) -> &Ident {
        match self {
            Decl::Data(Data { name, .. }) => name,
            Decl::Codata(Codata { name, .. }) => name,
            Decl::Def(Def { name, .. }) => name,
            Decl::Codef(Codef { name, .. }) => name,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Data {
    pub name: Ident,
    pub params: Telescope,
    pub ctors: Vec<Ctor>,
}

#[derive(Debug, Clone)]
pub struct Codata {
    pub name: Ident,
    pub params: Telescope,
    pub ctors: Vec<Dtor>,
}

#[derive(Debug, Clone)]
pub struct Ctor {
    pub name: Ident,
    pub params: Telescope,
    pub typ: TypApp,
}

#[derive(Debug, Clone)]
pub struct Dtor {
    pub name: Ident,
    pub params: Telescope,
    pub on_typ: TypApp,
    pub in_typ: Rc<Exp>,
}

#[derive(Debug, Clone)]
pub struct Def {
    pub name: Ident,
    pub params: Telescope,
    pub on_typ: TypApp,
    pub in_typ: Rc<Exp>,
    pub body: Match,
}

#[derive(Debug, Clone)]
pub struct Codef {
    pub name: Ident,
    pub params: Telescope,
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
    pub args: Telescope,
    pub body: Rc<Exp>,
}

#[derive(Debug, Clone)]
pub struct TypApp {
    pub name: Ident,
    pub subst: Subst,
}

#[derive(Debug, Clone)]
pub enum Exp {
    Var { var: Var },
    Ctor { name: Ident, subst: Subst },
    Dtor { exp: Rc<Exp>, name: Ident, subst: Subst },
    Ano { exp: Rc<Exp>, typ: Rc<Exp> },
    Type,
}

/// Wrapper type signifying the wrapped parameters have telescope
/// semantics. I.e. each parameter binding in the parameter list is in scope
/// for the following parameters.
#[derive(Debug, Clone)]
pub struct Telescope(pub Params);

pub type Params = Vec<Param>;
pub type Subst = Vec<Rc<Exp>>;

#[derive(Debug, Clone)]
pub struct Param {
    pub name: Ident,
    pub typ: Rc<Exp>,
}
