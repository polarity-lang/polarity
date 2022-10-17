use std::rc::Rc;

use data::HashMap;

use super::ast;
use super::common::*;

#[derive(Debug, Clone)]
pub struct Prg {
    pub map: HashMap<Ident, XData>,
    pub exp: Option<Rc<ast::Exp>>,
}

#[derive(Debug, Clone)]
pub struct XData {
    pub repr: Repr,
    pub info: ast::Info,
    pub name: Ident,
    pub typ: Rc<ast::TypAbs>,
    pub ctors: HashMap<Ident, ast::Ctor>,
    pub dtors: HashMap<Ident, ast::Dtor>,
    pub exprs: HashMap<Key, Option<Rc<ast::Exp>>>,
    pub impl_block: Option<ast::Impl>,
}

/// A key points to a matrix cell
///
/// The binding order in the matrix cell is as follors:
/// * dtor telescope
/// * ctor telescope
/// This invariant needs to be handled when translating
/// between the matrix and other representations
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Key {
    pub ctor: Ident,
    pub dtor: Ident,
}

#[derive(Debug, Clone, Copy)]
pub enum Repr {
    Data,
    Codata,
}
