use std::rc::Rc;

use codespan::Span;

use num_bigint::BigUint;

pub type Ident = String;

#[derive(Debug, Clone)]
pub struct DocComment {
    pub docs: Vec<String>,
}

/// An attribute can be attached to various nodes in the syntax tree.
/// We use the same syntax for attributes as Rust, that is `#[attr1,attr2]`.
#[derive(Debug, Clone, Default)]
pub struct Attribute {
    pub attrs: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum BindingSite {
    Var { name: Ident },
    Wildcard,
}

#[derive(Debug, Clone)]
pub struct Prg {
    pub items: Vec<Decl>,
    pub exp: Option<Rc<Exp>>,
}

#[derive(Debug, Clone)]
pub enum Decl {
    Data(Data),
    Codata(Codata),
    Def(Def),
    Codef(Codef),
}

#[derive(Debug, Clone)]
pub struct Data {
    pub span: Span,
    pub doc: Option<DocComment>,
    pub attr: Attribute,
    pub name: Ident,
    pub params: Telescope,
    pub ctors: Vec<Ctor>,
}

#[derive(Debug, Clone)]
pub struct Codata {
    pub span: Span,
    pub doc: Option<DocComment>,
    pub attr: Attribute,
    pub name: Ident,
    pub params: Telescope,
    pub dtors: Vec<Dtor>,
}

#[derive(Debug, Clone)]
pub struct Ctor {
    pub span: Span,
    pub doc: Option<DocComment>,
    pub name: Ident,
    pub params: Telescope,
    pub typ: Option<TypApp>,
}

#[derive(Debug, Clone)]
pub struct Dtor {
    pub span: Span,
    pub doc: Option<DocComment>,
    pub name: Ident,
    pub params: Telescope,
    pub destructee: Destructee,
    pub ret_typ: Rc<Exp>,
}

#[derive(Debug, Clone)]
pub struct Def {
    pub span: Span,
    pub doc: Option<DocComment>,
    pub name: Ident,
    pub attr: Attribute,
    pub params: Telescope,
    pub scrutinee: Scrutinee,
    pub ret_typ: Rc<Exp>,
    pub body: Match,
}

#[derive(Debug, Clone)]
pub struct Codef {
    pub span: Span,
    pub doc: Option<DocComment>,
    pub name: Ident,
    pub attr: Attribute,
    pub params: Telescope,
    pub typ: TypApp,
    pub body: Match,
}

#[derive(Debug, Clone)]
pub struct Match {
    pub span: Span,
    pub cases: Vec<Case>,
    pub omit_absurd: bool,
}

#[derive(Debug, Clone)]
pub struct Case {
    pub span: Span,
    pub name: Ident,
    pub args: TelescopeInst,
    /// Body being `None` represents an absurd pattern
    pub body: Option<Rc<Exp>>,
}

#[derive(Debug, Clone)]
pub struct Scrutinee {
    pub span: Span,
    pub name: Option<Ident>,
    pub typ: TypApp,
}

#[derive(Debug, Clone)]
pub struct Destructee {
    pub span: Span,
    pub name: Option<Ident>,
    pub typ: Option<TypApp>,
}

#[derive(Debug, Clone)]
pub struct SelfParam {
    pub span: Span,
    pub name: Option<Ident>,
    pub typ: TypApp,
}

impl From<Scrutinee> for SelfParam {
    fn from(scrutinee: Scrutinee) -> Self {
        let Scrutinee { span, name, typ } = scrutinee;

        SelfParam { span, name, typ }
    }
}

#[derive(Debug, Clone)]
pub struct TypApp {
    pub span: Span,
    pub name: Ident,
    pub args: Args,
}

impl TypApp {
    pub fn to_exp(&self) -> Exp {
        Exp::Call { span: self.span, name: self.name.clone(), args: self.args.clone() }
    }
}

#[derive(Debug, Clone)]
pub enum Exp {
    Call { span: Span, name: Ident, args: Args },
    DotCall { span: Span, exp: Rc<Exp>, name: Ident, args: Args },
    Anno { span: Span, exp: Rc<Exp>, typ: Rc<Exp> },
    Type { span: Span },
    Match { span: Span, name: Option<Ident>, on_exp: Rc<Exp>, motive: Option<Motive>, body: Match },
    Comatch { span: Span, name: Option<Ident>, is_lambda_sugar: bool, body: Match },
    Hole { span: Span },
    NatLit { span: Span, val: BigUint },
    Fun { span: Span, from: Rc<Exp>, to: Rc<Exp> },
    Lam { span: Span, var: ParamInst, body: Rc<Exp> },
}

#[derive(Debug, Clone)]
pub struct Motive {
    pub span: Span,
    pub param: ParamInst,
    pub ret_typ: Rc<Exp>,
}

/// Wrapper type signifying the wrapped parameters have telescope
/// semantics. I.e. each parameter binding in the parameter list is in scope
/// for the following parameters. This influences the lowering semantic.
#[derive(Debug, Clone)]
pub struct Telescope(pub Params);

/// Instantiation of a previously declared telescope
#[derive(Debug, Clone)]
pub struct TelescopeInst(pub Vec<ParamInst>);

impl Telescope {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

pub type Params = Vec<Param>;
pub type Args = Vec<Rc<Exp>>;

/// A `Param` can either be a single parameter, like `x : T`, or a list of parameters, like `x, y, z : T`.
#[derive(Debug, Clone)]
pub struct Param {
    /// The obligatory parameter.
    pub name: BindingSite,
    /// A possible list of additional parameters.
    pub names: Vec<BindingSite>,
    /// The type of the parameter.
    pub typ: Rc<Exp>,
}

/// Instantiation of a previously declared parameter
#[derive(Debug, Clone)]
pub struct ParamInst {
    pub span: Span,
    pub name: BindingSite,
}
