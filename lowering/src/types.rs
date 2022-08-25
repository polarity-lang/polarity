use super::ctx::*;
use super::result::*;

pub trait Lower {
    type Target;

    fn lower_in_ctx(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError>;
}

pub trait LowerTelescope {
    type Target;

    fn lower_telescope<T, F: Fn(&mut Ctx, Self::Target) -> Result<T, LoweringError>>(
        &self,
        ctx: &mut Ctx,
        f: F,
    ) -> Result<T, LoweringError>;
}

impl<T: LowerTelescope> Lower for T {
    type Target = <Self as LowerTelescope>::Target;

    fn lower_in_ctx(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        self.lower_telescope(ctx, |_, out| Ok(out))
    }
}
