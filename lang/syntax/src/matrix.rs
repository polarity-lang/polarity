use std::rc::Rc;

use data::HashMap;

use super::ust;
use crate::common::*;

#[derive(Debug, Clone)]
pub struct Prg {
    pub map: HashMap<Ident, XData>,
    pub exp: Option<Rc<ust::Exp>>,
}

#[derive(Debug, Clone)]
pub struct XData {
    pub repr: Repr,
    pub info: ust::Info,
    pub doc: Option<DocComment>,
    pub name: Ident,
    pub typ: Rc<ust::TypAbs>,
    pub ctors: HashMap<Ident, ust::Ctor>,
    pub dtors: HashMap<Ident, ust::Dtor>,
    pub exprs: HashMap<Key, Option<Rc<ust::Exp>>>,
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
