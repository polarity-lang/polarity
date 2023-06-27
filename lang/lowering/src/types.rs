use super::ctx::*;
use super::result::*;

pub trait Lower {
    type Target;

    fn lower_in_ctx(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError>;
}
