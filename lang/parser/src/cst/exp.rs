use std::rc::Rc;

use codespan::Span;
use num_bigint::BigUint;

use super::ident::*;

#[derive(Debug, Clone)]
pub enum BindingSite {
    Var { span: Span, name: Ident },
    Wildcard { span: Span },
}

#[derive(Debug, Clone)]
pub struct Pattern {
    pub name: Ident,
    pub params: Vec<BindingSite>,
}

#[derive(Debug, Clone)]
pub struct Copattern {
    pub name: Ident,
    pub params: Vec<BindingSite>,
}

#[derive(Debug, Clone)]
pub struct Case<P> {
    pub span: Span,
    pub pattern: P,
    /// Body being `None` represents an absurd pattern
    pub body: Option<Rc<Exp>>,
}

#[derive(Debug, Clone)]
pub enum Exp {
    Call(Call),
    DotCall(DotCall),
    Anno(Anno),
    TypeUniv(TypeUniv),
    LocalMatch(LocalMatch),
    LocalComatch(LocalComatch),
    Hole(Hole),
    NatLit(NatLit),
    Fun(Fun),
    Lam(Lam),
}

#[derive(Debug, Clone)]
pub struct Call {
    pub span: Span,
    pub name: Ident,
    pub args: Vec<Rc<Exp>>,
}

#[derive(Debug, Clone)]
pub struct DotCall {
    pub span: Span,
    pub exp: Rc<Exp>,
    pub name: Ident,
    pub args: Vec<Rc<Exp>>,
}

#[derive(Debug, Clone)]
pub struct Anno {
    pub span: Span,
    pub exp: Rc<Exp>,
    pub typ: Rc<Exp>,
}

#[derive(Debug, Clone)]
pub struct TypeUniv {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct LocalMatch {
    pub span: Span,
    pub name: Option<Ident>,
    pub on_exp: Rc<Exp>,
    pub motive: Option<Motive>,
    pub cases: Vec<Case<Pattern>>,
}

#[derive(Debug, Clone)]
pub struct LocalComatch {
    pub span: Span,
    pub name: Option<Ident>,
    pub is_lambda_sugar: bool,
    pub cases: Vec<Case<Copattern>>,
}

#[derive(Debug, Clone)]
pub struct Hole {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct NatLit {
    pub span: Span,
    pub val: BigUint,
}

#[derive(Debug, Clone)]
pub struct Fun {
    pub span: Span,
    pub from: Rc<Exp>,
    pub to: Rc<Exp>,
}

#[derive(Debug, Clone)]
pub struct Lam {
    pub span: Span,
    pub var: BindingSite,
    pub body: Rc<Exp>,
}

#[derive(Debug, Clone)]
pub struct Motive {
    pub span: Span,
    pub param: BindingSite,
    pub ret_typ: Rc<Exp>,
}
