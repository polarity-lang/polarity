use super::ctx::*;
use super::result::*;
mod decls;
mod exp;

pub trait Lower {
    type Target;

    fn lower(&self, ctx: &mut Ctx) -> LoweringResult<Self::Target>;
}

impl<T: Lower> Lower for Option<T> {
    type Target = Option<T::Target>;

    fn lower(&self, ctx: &mut Ctx) -> LoweringResult<Self::Target> {
        self.as_ref().map(|x| x.lower(ctx)).transpose()
    }
}

impl<T: Lower> Lower for Vec<T> {
    type Target = Vec<T::Target>;

    fn lower(&self, ctx: &mut Ctx) -> LoweringResult<Self::Target> {
        self.iter().map(|x| x.lower(ctx)).collect()
    }
}

impl<T: Lower> Lower for Box<T> {
    type Target = Box<T::Target>;

    fn lower(&self, ctx: &mut Ctx) -> LoweringResult<Self::Target> {
        Ok(Box::new((**self).lower(ctx)?))
    }
}
