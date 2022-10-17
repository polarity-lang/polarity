mod elab;

pub trait Forget {
    type Target;

    fn forget(&self) -> Self::Target;
}
