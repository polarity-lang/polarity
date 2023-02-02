use syntax::ast::*;
use syntax::common::*;
use syntax::ctx::*;

use crate::{Rename, RenameInfo};

use super::ctx::*;

impl<P: Phase<VarName = Ident>> Mapper<P> for Ctx
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
        self.ctx_map_telescope(params, f_acc, f_inner)
    }

    fn map_telescope_inst<X, I, F1, F2>(&mut self, params: I, f_acc: F1, f_inner: F2) -> X
    where
        I: IntoIterator<Item = ParamInst<P>>,
        F1: Fn(&mut Self, ParamInst<P>) -> ParamInst<P>,
        F2: FnOnce(&mut Self, TelescopeInst<P>) -> X,
    {
        self.ctx_map_telescope_inst(params, f_acc, f_inner)
    }

    fn map_self_param<X, F>(
        &mut self,
        info: <P as Phase>::Info,
        name: Option<Ident>,
        typ: TypApp<P>,
        f_inner: F,
    ) -> X
    where
        F: FnOnce(&mut Self, SelfParam<P>) -> X,
    {
        self.ctx_map_self_param(info, name, typ, f_inner)
    }

    fn map_motive_param<X, F>(&mut self, param: ParamInst<P>, f_inner: F) -> X
    where
        F: FnOnce(&mut Self, ParamInst<P>) -> X,
    {
        self.ctx_map_motive_param(param, f_inner)
    }

    fn map_exp_var(&mut self, info: P::TypeInfo, _name: P::VarName, idx: Idx) -> Exp<P> {
        Exp::Var { info, name: self.lookup(idx), idx }
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
    <Self as HasPhase>::Phase: Phase<VarName = Ident>,
    <<Self as HasPhase>::Phase as Phase>::TypeInfo: RenameInfo,
    <<Self as HasPhase>::Phase as Phase>::TypeAppInfo: RenameInfo,
{
    fn rename_in_ctx(self, ctx: &mut Ctx) -> Self {
        self.map(ctx)
    }
}
