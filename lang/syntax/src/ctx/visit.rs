use std::rc::Rc;

use crate::common::*;
use crate::ctx::*;
use crate::generic::*;
use codespan::Span;

pub trait VisitCtxExt<P: Phase> {
    fn ctx_visit_telescope<'a, I, F1, F2>(&mut self, params: I, f_acc: F1, f_inner: F2)
    where
        P: 'a,
        I: IntoIterator<Item = &'a Param<P>>,
        F1: Fn(&mut Self, &'a Param<P>),
        F2: FnOnce(&mut Self);

    fn ctx_visit_telescope_inst<'a, I, F1, F2>(&mut self, params: I, f_acc: F1, f_inner: F2)
    where
        P: 'a,
        I: IntoIterator<Item = &'a ParamInst<P>>,
        F1: Fn(&mut Self, &'a ParamInst<P>),
        F2: FnOnce(&mut Self);

    fn ctx_visit_motive_param<X, F>(&mut self, param: &ParamInst<P>, f_inner: F) -> X
    where
        F: FnOnce(&mut Self, &ParamInst<P>) -> X;

    fn ctx_visit_self_param<X, F>(
        &mut self,
        info: &Option<Span>,
        name: &Option<Ident>,
        typ: &TypApp<P>,
        f_inner: F,
    ) -> X
    where
        F: FnOnce(&mut Self) -> X;
}

impl<P: Phase, C: BindContext> VisitCtxExt<P> for C
where
    for<'a> &'a Param<P>: ContextElem<C::Ctx>,
    for<'a> &'a ParamInst<P>: ContextElem<C::Ctx>,
{
    fn ctx_visit_telescope<'a, I, F1, F2>(&mut self, params: I, f_acc: F1, f_inner: F2)
    where
        P: 'a,
        I: IntoIterator<Item = &'a Param<P>>,
        F1: Fn(&mut Self, &'a Param<P>),
        F2: FnOnce(&mut Self),
    {
        self.bind_fold(
            params.into_iter(),
            Vec::new(),
            |this, mut params_out, param| {
                f_acc(this, param);
                params_out.push(());
                params_out
            },
            |this, _params| f_inner(this),
        )
    }

    fn ctx_visit_telescope_inst<'a, I, F1, F2>(&mut self, params: I, f_acc: F1, f_inner: F2)
    where
        P: 'a,
        I: IntoIterator<Item = &'a ParamInst<P>>,
        F1: Fn(&mut Self, &'a ParamInst<P>),
        F2: FnOnce(&mut Self),
    {
        self.bind_fold(
            params.into_iter(),
            Vec::new(),
            |this, mut params_out, param| {
                f_acc(this, param);
                params_out.push(());
                params_out
            },
            |this, _params| f_inner(this),
        )
    }

    fn ctx_visit_motive_param<X, F>(&mut self, param: &ParamInst<P>, f_inner: F) -> X
    where
        F: FnOnce(&mut Self, &ParamInst<P>) -> X,
    {
        self.bind_single(param, |ctx| f_inner(ctx, param))
    }

    fn ctx_visit_self_param<X, F>(
        &mut self,
        _info: &Option<Span>,
        name: &Option<Ident>,
        typ: &TypApp<P>,
        f_inner: F,
    ) -> X
    where
        F: FnOnce(&mut Self) -> X,
    {
        let param = Param { name: name.clone().unwrap_or_default(), typ: Rc::new(typ.to_exp()) };

        self.bind_single(&param, |ctx| f_inner(ctx))
    }
}
