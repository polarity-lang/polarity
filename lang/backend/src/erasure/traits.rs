use super::ctx::Ctx;
use super::result::ErasureError;

pub trait Erasure {
    type Target;

    fn erase(&self, ctx: &mut Ctx) -> Result<Self::Target, ErasureError>;
}

impl<T: Erasure> Erasure for Vec<T> {
    type Target = Vec<T::Target>;

    fn erase(&self, ctx: &mut Ctx) -> Result<Self::Target, ErasureError> {
        self.iter().map(|x| x.erase(ctx)).collect()
    }
}

impl<T: Erasure> Erasure for Option<T> {
    type Target = Option<T::Target>;

    fn erase(&self, ctx: &mut Ctx) -> Result<Self::Target, ErasureError> {
        self.as_ref().map(|x| x.erase(ctx)).transpose()
    }
}
