use std::rc::Rc;

use codespan::Span;

use super::exp;

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
pub struct Prg {
    pub items: Vec<Decl>,
}

#[derive(Debug, Clone)]
pub enum Decl {
    Data(Data),
    Codata(Codata),
    Def(Def),
    Codef(Codef),
    Let(Let),
}

#[derive(Debug, Clone)]
pub struct Data {
    pub span: Span,
    pub doc: Option<DocComment>,
    pub attr: Attribute,
    pub name: exp::Ident,
    pub params: Telescope,
    pub ctors: Vec<Ctor>,
}

#[derive(Debug, Clone)]
pub struct Codata {
    pub span: Span,
    pub doc: Option<DocComment>,
    pub attr: Attribute,
    pub name: exp::Ident,
    pub params: Telescope,
    pub dtors: Vec<Dtor>,
}

#[derive(Debug, Clone)]
pub struct Ctor {
    pub span: Span,
    pub doc: Option<DocComment>,
    pub name: exp::Ident,
    pub params: Telescope,
    pub typ: Option<TypApp>,
}

#[derive(Debug, Clone)]
pub struct Dtor {
    pub span: Span,
    pub doc: Option<DocComment>,
    pub name: exp::Ident,
    pub params: Telescope,
    pub destructee: Destructee,
    pub ret_typ: Rc<exp::Exp>,
}

#[derive(Debug, Clone)]
pub struct Def {
    pub span: Span,
    pub doc: Option<DocComment>,
    pub name: exp::Ident,
    pub attr: Attribute,
    pub params: Telescope,
    pub scrutinee: Scrutinee,
    pub ret_typ: Rc<exp::Exp>,
    pub body: exp::Match,
}

#[derive(Debug, Clone)]
pub struct Codef {
    pub span: Span,
    pub doc: Option<DocComment>,
    pub name: exp::Ident,
    pub attr: Attribute,
    pub params: Telescope,
    pub typ: TypApp,
    pub body: exp::Match,
}

#[derive(Debug, Clone)]
pub struct Let {
    pub span: Span,
    pub doc: Option<DocComment>,
    pub name: exp::Ident,
    pub attr: Attribute,
    pub params: Telescope,
    pub typ: Rc<exp::Exp>,
    pub body: Rc<exp::Exp>,
}

#[derive(Debug, Clone)]
pub struct Scrutinee {
    pub span: Span,
    pub name: Option<exp::Ident>,
    pub typ: TypApp,
}

#[derive(Debug, Clone)]
pub struct Destructee {
    pub span: Span,
    pub name: Option<exp::Ident>,
    pub typ: Option<TypApp>,
}

#[derive(Debug, Clone)]
pub struct SelfParam {
    pub span: Span,
    pub name: Option<exp::Ident>,
    pub typ: TypApp,
}

#[derive(Debug, Clone)]
pub struct TypApp {
    pub span: Span,
    pub name: exp::Ident,
    pub args: exp::Args,
}

impl TypApp {
    pub fn to_exp(&self) -> exp::Exp {
        exp::Exp::Call { span: self.span, name: self.name.clone(), args: self.args.clone() }
    }
}

impl From<Scrutinee> for SelfParam {
    fn from(scrutinee: Scrutinee) -> Self {
        let Scrutinee { span, name, typ } = scrutinee;

        SelfParam { span, name, typ }
    }
}

/// A `Param` can either be a single parameter, like `x : T`, or a list of parameters, like `x, y, z : T`.
#[derive(Debug, Clone)]
pub struct Param {
    /// The obligatory parameter.
    pub name: exp::BindingSite,
    /// A possible list of additional parameters.
    pub names: Vec<exp::BindingSite>,
    /// The type of the parameter.
    pub typ: Rc<exp::Exp>,
}

/// Wrapper type signifying the wrapped parameters have telescope
/// semantics. I.e. each parameter binding in the parameter list is in scope
/// for the following parameters. This influences the lowering semantic.
#[derive(Debug, Clone)]
pub struct Telescope(pub Params);

impl Telescope {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

pub type Params = Vec<Param>;
