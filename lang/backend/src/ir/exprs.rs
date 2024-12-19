use codespan::Span;

use ast::{CallKind, DotCallKind, IdBound, Idx, VarBind, VarBound};

use super::kind::Kind;

#[derive(Debug, Clone)]
pub enum Exp {
    Variable(Variable),
    Call(Call),
    DotCall(DotCall),
    LocalMatch(LocalMatch),
    LocalComatch(LocalComatch),
    Hole(Hole),
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub span: Option<Span>,
    pub idx: Idx,
    pub name: VarBound,
    pub kind: Kind,
}

#[derive(Debug, Clone)]
pub struct Call {
    pub span: Option<Span>,
    pub call_kind: CallKind,
    pub name: IdBound,
    pub args: Args,
    pub ret_kind: Kind,
}

#[derive(Debug, Clone)]
pub struct DotCall {
    pub span: Option<Span>,
    pub kind: DotCallKind,
    pub exp: Box<Exp>,
    pub name: IdBound,
    pub args: Args,
    pub ret_kind: Kind,
}

#[derive(Debug, Clone)]
pub struct LocalMatch {
    pub span: Option<Span>,
    pub on_exp: Box<Exp>,
    pub cases: Vec<Case>,
    pub ret_kind: Kind,
}

#[derive(Debug, Clone)]
pub struct LocalComatch {
    pub span: Option<Span>,
    pub cases: Vec<Case>,
    pub ret_kind: Kind,
}

/// The only holes remaining are user-annotated holes `?` of kind `MetaVarKind::CanSolve`
/// Holes should compile to program aborts.
#[derive(Debug, Clone)]
pub struct Hole {
    pub span: Option<Span>,
}

#[derive(Debug, Clone)]
pub struct Case {
    pub span: Option<Span>,
    pub pattern: Pattern,
    pub body: Option<Box<Exp>>,
}

#[derive(Debug, Clone)]
pub struct Pattern {
    pub is_copattern: bool,
    pub name: IdBound,
    pub params: TelescopeInst,
}

#[derive(Debug, Clone)]
pub struct Args {
    pub args: Vec<Exp>,
}

#[derive(Debug, Clone)]
pub struct TelescopeInst {
    pub params: Vec<ParamInst>,
}

#[derive(Debug, Clone)]
pub struct ParamInst {
    pub span: Option<Span>,
    pub name: VarBind,
}
