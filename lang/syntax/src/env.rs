use std::rc::Rc;

use derivative::Derivative;

use crate::values::*;

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Env {
    pub bound: Vec<Vec<Rc<Val>>>,
}
