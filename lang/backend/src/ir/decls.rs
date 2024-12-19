//! High-level intermediate respresentation of the AST after erasure.
//! This representation is shared between any compiler backends and hence can only make few assumptions about the compilation target.

use codespan::Span;
use url::Url;

use ast::{Attributes, IdBind, UseDecl, VarBind};

use super::exprs::{Case, Exp};
use super::kind::Kind;

#[derive(Debug, Clone)]
pub struct Module {
    pub uri: Url,
    pub use_decls: Vec<UseDecl>,
    pub decls: Vec<Decl>,
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
    pub span: Option<Span>,
    pub name: IdBind,
    pub attr: Attributes,
    pub kind: Kind,
    pub ctors: Vec<Ctor>,
}

#[derive(Debug, Clone)]
pub struct Codata {
    pub span: Option<Span>,
    pub name: IdBind,
    pub attr: Attributes,
    pub kind: Kind,
    pub dtors: Vec<Dtor>,
}

#[derive(Debug, Clone)]
pub struct Ctor {
    pub span: Option<Span>,
    pub name: IdBind,
    pub params: Telescope,
    pub kind: Kind,
}

#[derive(Debug, Clone)]
pub struct Dtor {
    pub span: Option<Span>,
    pub name: IdBind,
    pub params: Telescope,
    pub self_param: SelfParam,
    pub ret_kind: Kind,
}

#[derive(Debug, Clone)]
pub struct Def {
    pub span: Option<Span>,
    pub name: IdBind,
    pub attr: Attributes,
    pub params: Telescope,
    pub self_param: SelfParam,
    pub ret_kind: Kind,
    pub cases: Vec<Case>,
}

#[derive(Debug, Clone)]
pub struct Codef {
    pub span: Option<Span>,
    pub name: IdBind,
    pub attr: Attributes,
    pub params: Telescope,
    pub kind: Kind,
    pub cases: Vec<Case>,
}

#[derive(Debug, Clone)]
pub struct Let {
    pub span: Option<Span>,
    pub name: IdBind,
    pub attr: Attributes,
    pub params: Telescope,
    pub kind: Kind,
    pub body: Box<Exp>,
}

#[derive(Debug, Clone)]
pub struct SelfParam {
    pub span: Option<Span>,
    pub name: Option<VarBind>,
    pub kind: Kind,
}

#[derive(Debug, Clone)]
pub struct Telescope {
    pub params: Vec<Param>,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: VarBind,
    pub kind: Kind,
}
