use std::rc::Rc;

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
    pub name: Ident,
    pub typ: TypAbs,
    pub ctors: Vec<Ident>,
}

#[derive(Debug, Clone)]
pub struct Codata {
    pub name: Ident,
    pub typ: TypAbs,
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
    // TODO: Consider renaming this field to "cocases"
    pub cases: Vec<Cocase>,
}

#[derive(Debug, Clone)]
pub struct Case {
    pub name: Ident,
    pub args: Telescope,
    pub eqns: Eqns,
    /// Body being `None` represents an absurd pattern
    pub body: Option<Rc<Exp>>,
}

#[derive(Debug, Clone)]
pub struct Cocase {
    pub name: Ident,
    pub args: Telescope,
    pub eqns: Eqns,
    /// Body being `None` represents an absurd pattern
    pub body: Option<Rc<Exp>>,
}

#[derive(Debug, Clone)]
pub struct Eqn {
    pub lhs: Rc<Exp>,
    pub rhs: Rc<Exp>,
}

#[derive(Debug, Clone)]
pub struct TypApp {
    pub name: Ident,
    pub args: Args,
}

#[derive(Debug, Clone)]
pub enum Exp {
    Var { info: Info, idx: Idx },
    TyCtor { info: Info, name: Ident, args: Args },
    Ctor { info: Info, name: Ident, args: Args },
    Dtor { info: Info, exp: Rc<Exp>, name: Ident, args: Args },
    Anno { info: Info, exp: Rc<Exp>, typ: Rc<Exp> },
    Type { info: Info },
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

#[derive(Debug, Clone)]
pub struct EqnParam {
    pub name: Ident,
    pub eqn: Eqn,
}

#[derive(Debug, Clone)]
pub struct Info {
    pub typ: Rc<ast::Exp>,
}

impl From<Rc<ast::Exp>> for Info {
    fn from(typ: Rc<ast::Exp>) -> Self {
        Info { typ }
    }
}

pub trait HasInfo {
    fn info(&self) -> &Info;
}

impl HasInfo for Exp {
    fn info(&self) -> &Info {
        match self {
            Exp::Var { info, .. } => info,
            Exp::TyCtor { info, .. } => info,
            Exp::Ctor { info, .. } => info,
            Exp::Dtor { info, .. } => info,
            Exp::Anno { info, .. } => info,
            Exp::Type { info } => info,
        }
    }
}

pub trait HasType {
    fn typ(&self) -> &Rc<ast::Exp>;
}

impl<T: HasInfo> HasType for T {
    fn typ(&self) -> &Rc<ast::Exp> {
        &self.info().typ
    }
}
