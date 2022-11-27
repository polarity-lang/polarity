use std::rc::Rc;

use derivative::Derivative;

use crate::values::*;

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Env {
    bound: Vec<Vec<Rc<Val>>>,
}

impl Env {
    pub(super) fn map<F>(&self, f: F) -> Self
    where
        F: Fn(&Rc<Val>) -> Rc<Val>,
    {
        let bound = self.bound.iter().map(|inner| inner.iter().map(&f).collect()).collect();
        Self { bound }
    }
}

impl From<Vec<Vec<Rc<Val>>>> for Env {
    fn from(bound: Vec<Vec<Rc<Val>>>) -> Self {
        Self { bound }
    }
}
