use syntax::common::*;
use syntax::de_bruijn::*;
use syntax::generic::*;

use crate::Rename;

use super::ctx::*;

impl<P: Phase<VarName = Ident>> Mapper<P> for Ctx {
    fn map_telescope<X, I, F1, F2>(&mut self, params: I, f_acc: F1, f_inner: F2) -> X
    where
        I: IntoIterator<Item = Param<P>>,
        F1: Fn(&mut Self, Param<P>) -> Param<P>,
        F2: FnOnce(&mut Self, Telescope<P>) -> X,
    {
        self.bind_fold(
            params.into_iter(),
            Vec::new(),
            |ctx, mut params_out, param| {
                params_out.push(f_acc(ctx, param));
                params_out
            },
            |ctx, params| {
                let params = Telescope { params };
                f_inner(ctx, params)
            },
        )
    }

    fn map_exp_var(&mut self, info: P::TypeInfo, _name: P::VarName, idx: Idx) -> Exp<P> {
        Exp::Var { info, name: self.bound(idx), idx }
    }
}

impl<T> Rename for T
where
    Self: HasPhase + Map<<Self as HasPhase>::Phase>,
    <Self as HasPhase>::Phase: Phase<VarName = Ident>,
{
    fn rename_in_ctx(self, ctx: &mut Ctx) -> Self {
        self.map(ctx)
    }
}
