//! High-level intermediate respresentation of the AST after erasure.
//! This representation is shared between any compiler backends and hence can only make few assumptions about the compilation target.

use url::Url;

use ast::UseDecl;

use super::exprs::{Case, Exp};

#[derive(Debug, Clone)]
pub struct Module {
    pub uri: Url,
    pub use_decls: Vec<UseDecl>,
    pub def_decls: Vec<Def>,
    pub codef_decls: Vec<Codef>,
    pub let_decls: Vec<Let>,
}

#[derive(Debug, Clone)]
pub struct Def {
    pub name: String,
    pub self_param: String,
    pub params: Vec<String>,
    pub cases: Vec<Case>,
}

#[derive(Debug, Clone)]
pub struct Codef {
    pub name: String,
    pub params: Vec<String>,
    pub cases: Vec<Case>,
}

#[derive(Debug, Clone)]
pub struct Let {
    pub name: String,
    pub params: Vec<String>,
    pub body: Box<Exp>,
}
