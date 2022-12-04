use std::rc::Rc;

pub trait Forget {
    type Target;

    fn forget(&self) -> Self::Target;
}

impl<T: Forget> Forget for Rc<T> {
    type Target = Rc<T::Target>;

    fn forget(&self) -> Self::Target {
        Rc::new(T::forget(self))
    }
}

impl<T: Forget> Forget for Option<T> {
    type Target = Option<T::Target>;

    fn forget(&self) -> Self::Target {
        self.as_ref().map(Forget::forget)
    }
}

impl<T: Forget> Forget for Vec<T> {
    type Target = Vec<T::Target>;

    fn forget(&self) -> Self::Target {
        self.iter().map(Forget::forget).collect()
    }
}
