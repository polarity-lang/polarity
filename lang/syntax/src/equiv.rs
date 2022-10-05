use std::rc::Rc;

pub trait AlphaEq {
    fn alpha_eq(&self, other: &Self) -> bool;
}

impl<T: AlphaEq> AlphaEq for Rc<T> {
    fn alpha_eq(&self, other: &Self) -> bool {
        (**self).alpha_eq(&**other)
    }
}
