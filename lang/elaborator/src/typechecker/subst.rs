use syntax::{
    ast::*,
    common::{Substitutable, Substitution},
    ctx::{BindContext, LevelCtx},
};

pub trait SubstUnderCtx: Substitutable {
    fn subst_under_ctx<S: Substitution>(&self, ctx: LevelCtx, s: &S) -> Self::Result;
}

impl<T: Substitutable + Clone> SubstUnderCtx for T {
    fn subst_under_ctx<S: Substitution>(&self, mut ctx: LevelCtx, s: &S) -> Self::Result {
        self.subst(&mut ctx, s)
    }
}

pub trait SubstInTelescope {
    /// Substitute in a telescope
    fn subst_in_telescope<S: Substitution>(&self, ctx: LevelCtx, s: &S) -> Self;
}

impl SubstInTelescope for Telescope {
    fn subst_in_telescope<S: Substitution>(&self, mut ctx: LevelCtx, s: &S) -> Self {
        let Telescope { params } = self;

        ctx.bind_fold(
            params.iter(),
            Vec::new(),
            |ctx, mut params_out, param| {
                params_out.push(param.subst(ctx, s));
                params_out
            },
            |_, params_out| Telescope { params: params_out },
        )
    }
}
