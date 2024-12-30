use super::ctx::Ctx;
use super::result::ErasureError;

pub trait Erasure {
    fn erase(&mut self, ctx: &Ctx) -> Result<(), ErasureError>;
}

impl<T: Erasure> Erasure for Vec<T> {
    fn erase(&mut self, ctx: &Ctx) -> Result<(), ErasureError> {
        for x in self {
            x.erase(ctx)?;
        }
        Ok(())
    }
}

impl<T: Erasure> Erasure for Option<T> {
    fn erase(&mut self, ctx: &Ctx) -> Result<(), ErasureError> {
        if let Some(x) = self {
            x.erase(ctx)?;
        }
        Ok(())
    }
}
