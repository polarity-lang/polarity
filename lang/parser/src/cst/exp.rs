use std::rc::Rc;

use codespan::Span;

use num_bigint::BigUint;

pub type Ident = String;

#[derive(Debug, Clone)]
pub enum BindingSite {
    Var { name: Ident },
    Wildcard,
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

pub type Args = Vec<Rc<Exp>>;

/// Instantiation of a previously declared parameter
#[derive(Debug, Clone)]
pub struct ParamInst {
    pub span: Span,
    pub name: BindingSite,
}

/// Instantiation of a previously declared telescope
#[derive(Debug, Clone)]
pub struct TelescopeInst(pub Vec<ParamInst>);
