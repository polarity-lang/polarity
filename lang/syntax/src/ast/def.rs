use std::rc::Rc;

use data::HashMap;

use crate::common::*;
use crate::de_bruijn::*;

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
        Self { map: HashMap::default(), order: Vec::new() }
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
    pub name: Ident,
    pub typ: Rc<TypAbs>,
    pub ctors: Vec<Ident>,
}

#[derive(Debug, Clone)]
pub struct Codata {
    pub name: Ident,
    pub typ: Rc<TypAbs>,
    pub dtors: Vec<Ident>,
}

#[derive(Debug, Clone)]
pub struct TypAbs {
    pub params: Telescope,
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

impl Def {
    pub fn to_dtor(&self) -> Dtor {
        Dtor {
            name: self.name.clone(),
            params: self.params.clone(),
            on_typ: self.on_typ.clone(),
            in_typ: self.in_typ.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Codef {
    pub name: Ident,
    pub params: Telescope,
    pub typ: TypApp,
    pub body: Comatch,
}

impl Codef {
    pub fn to_ctor(&self) -> Ctor {
        Ctor { name: self.name.clone(), params: self.params.clone(), typ: self.typ.clone() }
    }
}

#[derive(Debug, Clone)]
pub struct Match {
    pub cases: Vec<Case>,
}

#[derive(Debug, Clone)]
pub struct Comatch {
    pub cases: Vec<Cocase>,
}

#[derive(Debug, Clone)]
pub struct Case {
    pub name: Ident,
    pub args: Telescope,
    pub eqns: EqnParams,
    /// Body being `None` represents an absurd pattern
    pub body: Option<Rc<Exp>>,
}

#[derive(Debug, Clone)]
pub struct Cocase {
    pub name: Ident,
    pub args: Telescope,
    pub eqns: EqnParams,
    /// Body being `None` represents an absurd pattern
    pub body: Option<Rc<Exp>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Eqn {
    pub lhs: Rc<Exp>,
    pub rhs: Rc<Exp>,
}

impl From<(Rc<Exp>, Rc<Exp>)> for Eqn {
    fn from((lhs, rhs): (Rc<Exp>, Rc<Exp>)) -> Self {
        Eqn { lhs, rhs }
    }
}

#[derive(Debug, Clone)]
pub struct TypApp {
    pub name: Ident,
    pub args: Args,
}

impl TypApp {
    pub fn to_exp(&self) -> Exp {
        Exp::TypCtor { name: self.name.clone(), args: self.args.clone() }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Exp {
    Var { idx: Idx },
    TypCtor { name: Ident, args: Args },
    Ctor { name: Ident, args: Args },
    Dtor { exp: Rc<Exp>, name: Ident, args: Args },
    Anno { exp: Rc<Exp>, typ: Rc<Exp> },
    Type,
}

/// Wrapper type signifying the wrapped parameters have telescope
/// semantics. I.e. each parameter binding in the parameter list is in scope
/// for the following parameters.
#[derive(Debug, Clone)]
pub struct Telescope(pub Params);

impl Telescope {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

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
