use codespan::Span;
use parser::cst::Ident;
use syntax::common::*;
use syntax::ctx::*;
use syntax::generic::*;

use crate::{Rename, RenameInfo};

use super::ctx::*;

impl<P: Phase> Mapper<P> for Ctx
where
    P::TypeInfo: RenameInfo,
    P::TypeAppInfo: RenameInfo,
{
    fn map_telescope<X, I, F1, F2>(&mut self, params: I, f_acc: F1, f_inner: F2) -> X
    where
        I: IntoIterator<Item = Param<P>>,
        F1: Fn(&mut Self, Param<P>) -> Param<P>,
        F2: FnOnce(&mut Self, Telescope<P>) -> X,
    {
        self.bind_fold2(
            params.into_iter(),
            vec![],
            |ctx, mut acc, mut param| {
                param = f_acc(ctx, param);
                let new_name = ctx.disambiguate_name(param.name);
                param.name = new_name.clone();
                acc.push(param);
                BindElem { elem: new_name, ret: acc }
            },
            |ctx, params| f_inner(ctx, Telescope { params }),
        )
    }

    fn map_telescope_inst<X, I, F1, F2>(&mut self, params: I, f_acc: F1, f_inner: F2) -> X
    where
        I: IntoIterator<Item = ParamInst<P>>,
        F1: Fn(&mut Self, ParamInst<P>) -> ParamInst<P>,
        F2: FnOnce(&mut Self, TelescopeInst<P>) -> X,
    {
        self.bind_fold2(
            params.into_iter(),
            vec![],
            |ctx, mut acc, mut param| {
                param = f_acc(ctx, param);
                let new_name = ctx.disambiguate_name(param.name);
                param.name = new_name.clone();
                acc.push(param);
                BindElem { elem: new_name, ret: acc }
            },
            |ctx, params| f_inner(ctx, TelescopeInst { params }),
        )
    }

    fn map_self_param<X, F>(
        &mut self,
        info: Option<Span>,
        name: Option<Ident>,
        typ: TypApp<P>,
        f_inner: F,
    ) -> X
    where
        F: FnOnce(&mut Self, SelfParam<P>) -> X,
    {
        self.ctx_map_self_param(info, name.map(|name| self.disambiguate_name(name)), typ, f_inner)
    }

    fn map_motive_param<X, F>(&mut self, mut param: ParamInst<P>, f_inner: F) -> X
    where
        F: FnOnce(&mut Self, ParamInst<P>) -> X,
    {
        param.name = self.disambiguate_name(param.name);
        self.ctx_map_motive_param(param, f_inner)
    }

    fn map_exp_var(&mut self, info: P::TypeInfo, _name: Ident, ctx: P::Ctx, idx: Idx) -> Exp<P> {
        Exp::Var { info, name: self.lookup(idx), ctx, idx }
    }

    fn map_type_info(&mut self, info: <P as Phase>::TypeInfo) -> <P as Phase>::TypeInfo {
        info.rename_in_ctx(self)
    }

    fn map_type_app_info(&mut self, info: <P as Phase>::TypeAppInfo) -> <P as Phase>::TypeAppInfo {
        info.rename_in_ctx(self)
    }
}

impl<T> Rename for T
where
    Self: HasPhase + Map<<Self as HasPhase>::Phase>,
    <Self as HasPhase>::Phase: Phase,
    <<Self as HasPhase>::Phase as Phase>::TypeInfo: RenameInfo,
    <<Self as HasPhase>::Phase as Phase>::TypeAppInfo: RenameInfo,
{
    fn rename_in_ctx(self, ctx: &mut Ctx) -> Self {
        self.map(ctx)
    }
}
