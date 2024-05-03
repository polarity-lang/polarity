use crate::ctx::*;
use crate::generic::*;
use codespan::Span;

use std::rc::Rc;

pub trait MapCtxExt {
    fn ctx_map_telescope<X, I, F1, F2>(&mut self, params: I, f_acc: F1, f_inner: F2) -> X
    where
        I: IntoIterator<Item = Param>,
        F1: Fn(&mut Self, Param) -> Param,
        F2: FnOnce(&mut Self, Telescope) -> X;

    fn ctx_map_telescope_inst<X, I, F1, F2>(&mut self, params: I, f_acc: F1, f_inner: F2) -> X
    where
        I: IntoIterator<Item = ParamInst>,
        F1: Fn(&mut Self, ParamInst) -> ParamInst,
        F2: FnOnce(&mut Self, TelescopeInst) -> X;

    fn ctx_map_motive_param<X, F>(&mut self, param: ParamInst, f_inner: F) -> X
    where
        F: FnOnce(&mut Self, ParamInst) -> X;

    fn ctx_map_self_param<X, F>(
        &mut self,
        info: Option<Span>,
        name: Option<Ident>,
        typ: TypCtor,
        f_inner: F,
    ) -> X
    where
        F: FnOnce(&mut Self, SelfParam) -> X;
}

impl<C: BindContext> MapCtxExt for C
where
    Param: ContextElem<C::Ctx>,
    ParamInst: ContextElem<C::Ctx>,
{
    fn ctx_map_telescope<X, I, F1, F2>(&mut self, params: I, f_acc: F1, f_inner: F2) -> X
    where
        I: IntoIterator<Item = Param>,
        F1: Fn(&mut Self, Param) -> Param,
        F2: FnOnce(&mut Self, Telescope) -> X,
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
        I: IntoIterator<Item = ParamInst>,
        F1: Fn(&mut Self, ParamInst) -> ParamInst,
        F2: FnOnce(&mut Self, TelescopeInst) -> X,
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

    fn ctx_map_motive_param<X, F>(&mut self, param: ParamInst, f_inner: F) -> X
    where
        F: FnOnce(&mut Self, ParamInst) -> X,
    {
        self.bind_single(param.clone(), |ctx| f_inner(ctx, param))
    }

    fn ctx_map_self_param<X, F>(
        &mut self,
        info: Option<Span>,
        name: Option<Ident>,
        typ: TypCtor,
        f_inner: F,
    ) -> X
    where
        F: FnOnce(&mut Self, SelfParam) -> X,
    {
        let param = Param { name: name.clone().unwrap_or_default(), typ: Rc::new(typ.to_exp()) };
        self.bind_single(param, |ctx| f_inner(ctx, SelfParam { info, name, typ }))
    }
}
