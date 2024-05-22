//! Variable context
//!
//! Tracks locally bound variables

use std::rc::Rc;

use crate::ast::traits::Shift;
use crate::ast::*;

use super::context::GenericContext;

pub type TypeCtx = GenericContext<Binder>;

impl TypeCtx {
    pub fn map_failable<E, F>(&self, f: F) -> Result<Self, E>
    where
        F: Fn(&Rc<Exp>) -> Result<Rc<Exp>, E>,
    {
        let bound: Result<_, _> = self
            .bound
            .iter()
            .map(|stack| {
                stack.iter().map(|b| Ok(Binder { name: b.name.clone(), typ: f(&b.typ)? })).collect()
            })
            .collect();

        Ok(Self { bound: bound? })
    }
}

#[derive(Debug, Clone)]
pub struct Binder {
    pub name: Ident,
    pub typ: Rc<Exp>,
}

impl Shift for Binder {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        Self { name: self.name.clone(), typ: self.typ.shift_in_range(range, by) }
    }
}
