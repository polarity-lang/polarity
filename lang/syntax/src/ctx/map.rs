use crate::ctx::*;
use crate::generic::*;
use parser::cst::Ident;

use std::rc::Rc;

pub trait MapCtxExt<P: Phase> {
    fn ctx_map_telescope<X, I, F1, F2>(&mut self, params: I, f_acc: F1, f_inner: F2) -> X
    where
        I: IntoIterator<Item = Param<P>>,
        F1: Fn(&mut Self, Param<P>) -> Param<P>,
        F2: FnOnce(&mut Self, Telescope<P>) -> X;

    fn ctx_map_telescope_inst<X, I, F1, F2>(&mut self, params: I, f_acc: F1, f_inner: F2) -> X
    where
        I: IntoIterator<Item = ParamInst<P>>,
        F1: Fn(&mut Self, ParamInst<P>) -> ParamInst<P>,
        F2: FnOnce(&mut Self, TelescopeInst<P>) -> X;

    fn ctx_map_motive_param<X, F>(&mut self, param: ParamInst<P>, f_inner: F) -> X
    where
        F: FnOnce(&mut Self, ParamInst<P>) -> X;

    fn ctx_map_self_param<X, F>(
        &mut self,
        info: P::Info,
        name: Option<Ident>,
        typ: TypApp<P>,
        f_inner: F,
    ) -> X
    where
        F: FnOnce(&mut Self, SelfParam<P>) -> X;
}

impl<P: Phase, C: BindContext> MapCtxExt<P> for C
where
    Param<P>: ContextElem<C::Ctx>,
    ParamInst<P>: ContextElem<C::Ctx>,
{
    fn ctx_map_telescope<X, I, F1, F2>(&mut self, params: I, f_acc: F1, f_inner: F2) -> X
    where
        I: IntoIterator<Item = Param<P>>,
        F1: Fn(&mut Self, Param<P>) -> Param<P>,
        F2: FnOnce(&mut Self, Telescope<P>) -> X,
    {
        self.bind_fold(
            params.into_iter(),
            Vec::new(),
            |this, mut params_out, param| {
                params_out.push(f_acc(this, param));
                params_out
            },
            |this, params| {
                let telescope = Telescope { params };
                f_inner(this, telescope)
            },
        )
    }

    fn ctx_map_telescope_inst<X, I, F1, F2>(&mut self, params: I, f_acc: F1, f_inner: F2) -> X
    where
        I: IntoIterator<Item = ParamInst<P>>,
        F1: Fn(&mut Self, ParamInst<P>) -> ParamInst<P>,
        F2: FnOnce(&mut Self, TelescopeInst<P>) -> X,
    {
        self.bind_fold(
            params.into_iter(),
            Vec::new(),
            |this, mut params_out, param| {
                params_out.push(f_acc(this, param));
                params_out
            },
            |this, params| {
                let telescope_inst = TelescopeInst { params };
                f_inner(this, telescope_inst)
            },
        )
    }

    fn ctx_map_motive_param<X, F>(&mut self, param: ParamInst<P>, f_inner: F) -> X
    where
        F: FnOnce(&mut Self, ParamInst<P>) -> X,
    {
        self.bind_single(param.clone(), |ctx| f_inner(ctx, param))
    }

    fn ctx_map_self_param<X, F>(
        &mut self,
        info: P::Info,
        name: Option<Ident>,
        typ: TypApp<P>,
        f_inner: F,
    ) -> X
    where
        F: FnOnce(&mut Self, SelfParam<P>) -> X,
    {
        let param = Param { name: name.clone().unwrap_or_default(), typ: Rc::new(typ.to_exp()) };
        self.bind_single(param, |ctx| f_inner(ctx, SelfParam { info, name, typ }))
    }
}
