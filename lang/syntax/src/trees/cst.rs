use std::rc::Rc;

use codespan::ByteIndex;
use codespan::Span;

use num_bigint::BigUint;

use crate::common::*;

pub type Ident = String;

#[derive(Debug, Clone)]
pub struct Prg {
    pub items: Vec<Item>,
    pub exp: Option<Rc<Exp>>,
}

#[derive(Debug, Clone)]
pub enum Item {
    Data(Data),
    Codata(Codata),
    Def(Def),
    Codef(Codef),
}

#[derive(Debug, Clone)]
pub struct Data {
    pub info: Info,
    pub doc: Option<DocComment>,
    /// Whether the declaration should be omitted
    /// during pretty printing.
    pub hidden: bool,
    pub name: Ident,
    pub params: Telescope,
    pub ctors: Vec<Ctor>,
}

#[derive(Debug, Clone)]
pub struct Codata {
    pub info: Info,
    pub doc: Option<DocComment>,
    /// Whether the declaration should be omitted
    /// during pretty printing.
    pub hidden: bool,
    pub name: Ident,
    pub params: Telescope,
    pub dtors: Vec<Dtor>,
}

#[derive(Debug, Clone)]
pub struct Ctor {
    pub info: Info,
    pub doc: Option<DocComment>,
    pub name: Ident,
    pub params: Telescope,
    pub typ: Option<TypApp>,
}

#[derive(Debug, Clone)]
pub struct Dtor {
    pub info: Info,
    pub doc: Option<DocComment>,
    pub name: Ident,
    pub params: Telescope,
    pub destructee: Destructee,
    pub ret_typ: Rc<Exp>,
}

#[derive(Debug, Clone)]
pub struct Def {
    pub info: Info,
    pub doc: Option<DocComment>,
    pub name: Ident,
    /// Whether the declaration should be omitted
    /// during pretty printing.
    pub hidden: bool,
    pub params: Telescope,
    pub scrutinee: Scrutinee,
    pub ret_typ: Rc<Exp>,
    pub body: Match,
}

#[derive(Debug, Clone)]
pub struct Codef {
    pub info: Info,
    pub doc: Option<DocComment>,
    pub name: Ident,
    /// Whether the declaration should be omitted
    /// during pretty printing.
    pub hidden: bool,
    pub params: Telescope,
    pub typ: TypApp,
    pub body: Comatch,
}

#[derive(Debug, Clone)]
pub struct Match {
    pub info: Info,
    pub cases: Vec<Case>,
    pub omit_absurd: bool,
}

#[derive(Debug, Clone)]
pub struct Comatch {
    pub info: Info,
    pub cases: Vec<Cocase>,
    pub omit_absurd: bool,
}

#[derive(Debug, Clone)]
pub struct Case {
    pub info: Info,
    pub name: Ident,
    pub args: TelescopeInst,
    /// Body being `None` represents an absurd pattern
    pub body: Option<Rc<Exp>>,
}

#[derive(Debug, Clone)]
pub struct Cocase {
    pub info: Info,
    pub name: Ident,
    pub args: TelescopeInst,
    /// Body being `None` represents an absurd pattern
    pub body: Option<Rc<Exp>>,
}

#[derive(Debug, Clone)]
pub struct Scrutinee {
    pub info: Info,
    pub name: Option<Ident>,
    pub typ: TypApp,
}

#[derive(Debug, Clone)]
pub struct Destructee {
    pub info: Info,
    pub name: Option<Ident>,
    pub typ: Option<TypApp>,
}

#[derive(Debug, Clone)]
pub struct SelfParam {
    pub info: Info,
    pub name: Option<Ident>,
    pub typ: TypApp,
}

impl From<Scrutinee> for SelfParam {
    fn from(scrutinee: Scrutinee) -> Self {
        let Scrutinee { info, name, typ } = scrutinee;

        SelfParam { info, name, typ }
    }
}

#[derive(Debug, Clone)]
pub struct TypApp {
    pub info: Info,
    pub name: Ident,
    pub args: Args,
}

impl TypApp {
    pub fn to_exp(&self) -> Exp {
        Exp::Call { info: self.info.clone(), name: self.name.clone(), args: self.args.clone() }
    }
}

#[derive(Debug, Clone)]
pub enum Exp {
    Call { info: Info, name: Ident, args: Args },
    DotCall { info: Info, exp: Rc<Exp>, name: Ident, args: Args },
    Anno { info: Info, exp: Rc<Exp>, typ: Rc<Exp> },
    Type { info: Info },
    Match { info: Info, name: Option<Ident>, on_exp: Rc<Exp>, motive: Option<Motive>, body: Match },
    Comatch { info: Info, name: Option<Ident>, is_lambda_sugar: bool, body: Comatch },
    Hole { info: Info, kind: HoleKind },
    NatLit { info: Info, val: BigUint },
    Fun { info: Info, from: Rc<Exp>, to: Rc<Exp> },
    Lam { info: Info, var: ParamInst, body: Rc<Exp> },
}

#[derive(Debug, Clone)]
pub struct Motive {
    pub info: Info,
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
    pub info: Info,
    pub name: BindingSite,
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
