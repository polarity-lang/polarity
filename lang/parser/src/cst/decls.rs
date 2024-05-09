use std::rc::Rc;

use codespan::Span;
use url::Url;

use super::exp;
use super::ident::*;

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
pub struct Module {
    /// The location of the module on disk
    pub uri: Url,
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

/// Data type declaration
///
/// ```text
/// data F(...) { ... }
///      ^  ^      ^----- ctors
///      |  \------------ params
///      \--------------- name
/// ```
#[derive(Debug, Clone)]
pub struct Data {
    pub span: Span,
    pub doc: Option<DocComment>,
    pub attr: Attribute,
    pub name: Ident,
    pub params: Telescope,
    pub ctors: Vec<Ctor>,
}

/// Codata type declaration
///
/// ```text
/// codata F(...) { ... }
///        ^  ^      ^----- ctors
///        |  \------------ params
///        \--------------- name
/// ```
#[derive(Debug, Clone)]
pub struct Codata {
    pub span: Span,
    pub doc: Option<DocComment>,
    pub attr: Attribute,
    pub name: Ident,
    pub params: Telescope,
    pub dtors: Vec<Dtor>,
}

/// Declaration of a constructor within the context of a data type declaration.
///
/// ```text
/// data F(...) { C(...) : F(...) }
///               ^  ^     ^^^^^^
///               |  |       \---- typ
///               |  \------------ params
///               \--------------- name
/// ```
/// The `typ` of the constructor is optional.
#[derive(Debug, Clone)]
pub struct Ctor {
    pub span: Span,
    pub doc: Option<DocComment>,
    pub name: Ident,
    pub params: Telescope,
    pub typ: Option<TypApp>,
}

/// Declaration of a destructor within the context of a codata type declaration.
///
/// ```text
/// codata F(...) { (self: F(...)).d(...) : t }
///                 ^^^^^^^^^^^^^^ ^  ^     ^
///                       |        |  |     \----- ret_typ
///                       |        |  \----------- params
///                       |        \-------------- name
///                       \----------------------- destructee
/// ```
#[derive(Debug, Clone)]
pub struct Dtor {
    pub span: Span,
    pub doc: Option<DocComment>,
    pub name: Ident,
    pub params: Telescope,
    pub destructee: Destructee,
    pub ret_typ: Rc<exp::Exp>,
}

/// Destructee within the context of a destructor declaration in a codata type.
///
/// ```text
/// codata F(...) { (self: F(...)).d(...) : t }
///                  ^^^^  ^^^^^^
///                    |      \----- typ
///                    \------------ name
/// ```
#[derive(Debug, Clone)]
pub struct Destructee {
    pub span: Span,
    pub name: Option<Ident>,
    pub typ: Option<TypApp>,
}

/// Toplevel definition, i.e. a global pattern match.
///
/// ```text
/// def (self: F(...)).d(...) : t { ... }
///     ^^^^^^^^^^^^^^ ^  ^     ^    ^
///            |       |  |     |    \----- body
///            |       |  |     \---------- ret_typ
///            |       |  \---------------- params
///            |       \------------------- name
///            \--------------------------- scrutinee
/// ```
#[derive(Debug, Clone)]
pub struct Def {
    pub span: Span,
    pub doc: Option<DocComment>,
    pub name: Ident,
    pub attr: Attribute,
    pub params: Telescope,
    pub scrutinee: Scrutinee,
    pub ret_typ: Rc<exp::Exp>,
    pub body: exp::Match,
}

/// Scrutinee within a toplevel definition
///
/// ```text
/// def (self: F(...)).d(...) : t { ... }
///      ^^^^  ^^^^^^
///        |      \----- typ
///        \------------ name
/// ```
#[derive(Debug, Clone)]
pub struct Scrutinee {
    pub span: Span,
    pub name: Option<Ident>,
    pub typ: TypApp,
}

/// Toplevel codefinition, i.e. a global copattern match.
///
/// ```text
/// codef C(...) : F(...) { ... }
///       ^  ^     ^^^^^^    ^
///       |  |        |      \------ body
///       |  |        \------------- typ
///       |  \---------------------- params
///       \------------------------- name
/// ```
#[derive(Debug, Clone)]
pub struct Codef {
    pub span: Span,
    pub doc: Option<DocComment>,
    pub name: Ident,
    pub attr: Attribute,
    pub params: Telescope,
    pub typ: TypApp,
    pub body: exp::Match,
}

/// Toplevel let-bound expression.
///
/// ```text
/// let f(...) : t { e }
///     ^  ^     ^   ^
///     |  |     |   \---- body
///     |  |     \-------- typ
///     |  \-------------- params
///     \----------------- name
/// ```
#[derive(Debug, Clone)]
pub struct Let {
    pub span: Span,
    pub doc: Option<DocComment>,
    pub name: Ident,
    pub attr: Attribute,
    pub params: Telescope,
    pub typ: Rc<exp::Exp>,
    pub body: Rc<exp::Exp>,
}

/// A typconstructor applied to arguments.
#[derive(Debug, Clone)]
pub struct TypApp {
    pub span: Span,
    pub name: Ident,
    pub args: Vec<Rc<exp::Exp>>,
}

impl TypApp {
    pub fn to_exp(&self) -> exp::Exp {
        exp::Exp::Call(exp::Call {
            span: self.span,
            name: self.name.clone(),
            args: self.args.clone(),
        })
    }
}

/// A `Param` can either be a single parameter, like `x : T`, or a list of parameters, like `x, y, z : T`.
#[derive(Debug, Clone)]
pub struct Param {
    /// The obligatory parameter name.
    pub name: exp::BindingSite,
    /// A possible list of additional parameter names.
    pub names: Vec<exp::BindingSite>,
    /// The type of the parameter(s).
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
