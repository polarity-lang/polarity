use std::rc::Rc;

use codespan::Span;

use crate::common::*;

use super::def::*;
use super::lookup_table::LookupTable;

#[rustfmt::skip]
#[allow(unused_variables)]
#[allow(clippy::too_many_arguments)]
pub trait Visitor<P: Phase> {
    fn visit_prg(&mut self, decls: &Decls<P>) {}
    fn visit_decls(&mut self, map: &HashMap<Ident, Decl<P>>, source: &LookupTable) {}
    fn visit_decl(&mut self, decl: &Decl<P>) {}
    fn visit_decl_data(&mut self, data: &Data<P>) {}
    fn visit_decl_codata(&mut self, codata: &Codata<P>) {}
    fn visit_decl_ctor(&mut self, ctor: &Ctor<P>) {}
    fn visit_decl_dtor(&mut self, dtor: &Dtor<P>) {}
    fn visit_decl_def(&mut self, def: &Def<P>) {}
    fn visit_decl_codef(&mut self, codef: &Codef<P>) {}
    fn visit_decl_let(&mut self, tl_let: &Let<P>) {}
    fn visit_data(&mut self, info: &Option<Span>, doc: &Option<DocComment>, name: &Ident, attr: &Attribute, typ: &Rc<TypAbs<P>>, ctors: &[Ident]) {}
    fn visit_codata(&mut self, info: &Option<Span>, doc: &Option<DocComment>, name: &Ident, attr: &Attribute,  typ: &Rc<TypAbs<P>>, dtors: &[Ident]) {}
    fn visit_typ_abs(&mut self, params: &Telescope<P>) {}
    fn visit_ctor(&mut self, info: &Option<Span>, doc: &Option<DocComment>, name: &Ident, params: &Telescope<P>, typ: &TypApp<P>) {}
    fn visit_dtor(&mut self, info: &Option<Span>, doc: &Option<DocComment>, name: &Ident, params: &Telescope<P>, self_param: &SelfParam<P>, ret_typ: &Rc<Exp<P>>) {}
    fn visit_def(&mut self, info: &Option<Span>, doc: &Option<DocComment>, name: &Ident, attr: &Attribute, params: &Telescope<P>, self_param: &SelfParam<P>, ret_typ: &Rc<Exp<P>>, body: &Match<P>) {}
    fn visit_codef(&mut self, info: &Option<Span>, doc: &Option<DocComment>, name: &Ident, attr: &Attribute, params: &Telescope<P>, typ: &TypApp<P>, body: &Match<P>) {}
    fn visit_let(&mut self, info: &Option<Span>, doc: &Option<DocComment>, name: &Ident, attr: &Attribute, params: &Telescope<P>, typ: &Rc<Exp<P>>, body: &Rc<Exp<P>>) {}
    fn visit_match(&mut self, info: &Option<Span>, cases: &[Case<P>], omit_absurd: bool) {}
    fn visit_case(&mut self, info: &Option<Span>, name: &Ident, args: &TelescopeInst<P>, body: &Option<Rc<Exp<P>>>) {}
    fn visit_typ_app(&mut self, info: &P::TypeInfo, name: &Ident, args: &Args<P>) {}
    fn visit_exp_var(&mut self, info: &P::TypeInfo, name: &Ident, ctx: &P::Ctx, idx: &Idx) {}
    fn visit_exp_typ_ctor(&mut self, info: &P::TypeInfo, name: &Ident, args: &Args<P>) {}
    fn visit_exp_ctor(&mut self, info: &P::TypeInfo, name: &Ident, args: &Args<P>) {}
    fn visit_exp_dtor(&mut self, info: &P::TypeInfo, exp: &Rc<Exp<P>>, name: &Ident, args: &Args<P>) {}
    fn visit_exp_anno(&mut self, info: &P::TypeInfo, exp: &Rc<Exp<P>>, typ: &Rc<Exp<P>>) {}
    fn visit_exp_type(&mut self, info: &P::TypeInfo) {}
    fn visit_exp_match(&mut self, info: &P::TypeAppInfo, ctx: &P::Ctx, name: &Label, on_exp: &Rc<Exp<P>>, ret_typ: &P::InfTyp, body: &Match<P>) {}
    fn visit_exp_comatch(&mut self, info: &P::TypeAppInfo, ctx: &P::Ctx, name: &Label, is_lambda_sugar: &bool, body: &Match<P>) {}
    fn visit_exp_hole(&mut self, info: &P::TypeInfo) {}
    fn visit_motive(&mut self, info: &Option<Span>, param: &ParamInst<P>, ret_typ: &Rc<Exp<P>>) {}
    fn visit_motive_param<X, F>(&mut self, param: &ParamInst<P>, f_inner: F) -> X
    where
        F: FnOnce(&mut Self, &ParamInst<P>) -> X
    {
        f_inner(self, param)
    }
    fn visit_telescope<'a, I, F1, F2>(&mut self, params: I, f_acc: F1, f_inner: F2)
    where
        P: 'a,
        I: IntoIterator<Item=&'a Param<P>>,
        F1: Fn(&mut Self, &'a Param<P>),
        F2: FnOnce(&mut Self)
    {
        for param in params.into_iter() {
            f_acc(self, param);
        }
        f_inner(self)
    }
    fn visit_telescope_inst<'a, I, F1, F2>(&mut self, params: I, f_acc: F1, f_inner: F2)
    where
        P: 'a,
        I: IntoIterator<Item=&'a ParamInst<P>>,
        F1: Fn(&mut Self, &'a ParamInst<P>),
        F2: FnOnce(&mut Self)
    {
        for param in params.into_iter() {
            f_acc(self, param);
        }
        f_inner(self)
    }
    fn visit_self_param<X, F>(&mut self, info: &Option<Span>, name: &Option<Ident>, typ: &TypApp<P>, f_inner: F) -> X
    where
        F: FnOnce(&mut Self) -> X
    {
        f_inner(self)
    }
    fn visit_param(&mut self, name: &Ident, typ: &Rc<Exp<P>>) {}
    fn visit_param_inst(&mut self, info: &P::TypeInfo, name: &Ident, typ: &P::InfTyp) {}
    fn visit_info(&mut self, info: &Option<Span>) {}
    fn visit_type_info(&mut self, info: &P::TypeInfo) {}
    fn visit_type_app_info(&mut self, info: &P::TypeAppInfo) {}
    fn visit_idx(&mut self, idx: &Idx) {}
    fn visit_typ(&mut self, typ: &P::InfTyp) {}
    fn visit_ctx(&mut self, ctx: &P::Ctx) {}
}

pub trait Visit<P: Phase> {
    fn visit<V>(&self, v: &mut V)
    where
        V: Visitor<P>;
}

impl<P: Phase, T: Visit<P> + Clone> Visit<P> for Rc<T> {
    fn visit<V>(&self, v: &mut V)
    where
        V: Visitor<P>,
    {
        T::visit(self, v)
    }
}

impl<P: Phase, T: Visit<P>> Visit<P> for Option<T> {
    fn visit<V>(&self, v: &mut V)
    where
        V: Visitor<P>,
    {
        if let Some(inner) = self {
            inner.visit(v);
        }
    }
}

impl<P: Phase, T: Visit<P>> Visit<P> for Vec<T> {
    fn visit<V>(&self, v: &mut V)
    where
        V: Visitor<P>,
    {
        for inner in self.iter() {
            inner.visit(v);
        }
    }
}

impl<P: Phase> Visit<P> for Prg<P> {
    fn visit<V>(&self, v: &mut V)
    where
        V: Visitor<P>,
    {
        let Prg { decls } = self;
        decls.visit(v);
        v.visit_prg(decls)
    }
}

impl<P: Phase> Visit<P> for Decls<P> {
    fn visit<V>(&self, v: &mut V)
    where
        V: Visitor<P>,
    {
        let Decls { map, lookup_table } = self;
        for decl in map.values() {
            decl.visit(v)
        }
        v.visit_decls(map, lookup_table)
    }
}

impl<P: Phase> Visit<P> for Decl<P> {
    fn visit<V>(&self, v: &mut V)
    where
        V: Visitor<P>,
    {
        match self {
            Decl::Data(inner) => {
                inner.visit(v);
                v.visit_decl_data(inner)
            }
            Decl::Codata(inner) => {
                inner.visit(v);
                v.visit_decl_codata(inner)
            }
            Decl::Ctor(inner) => {
                inner.visit(v);
                v.visit_decl_ctor(inner)
            }
            Decl::Dtor(inner) => {
                inner.visit(v);
                v.visit_decl_dtor(inner)
            }
            Decl::Def(inner) => {
                inner.visit(v);
                v.visit_decl_def(inner)
            }
            Decl::Codef(inner) => {
                inner.visit(v);
                v.visit_decl_codef(inner)
            }
            Decl::Let(inner) => {
                inner.visit(v);
                v.visit_decl_let(inner)
            }
        }
        v.visit_decl(self)
    }
}

impl<P: Phase> Visit<P> for Data<P> {
    fn visit<V>(&self, v: &mut V)
    where
        V: Visitor<P>,
    {
        let Data { info, doc, name, attr, typ, ctors } = self;
        typ.visit(v);
        v.visit_info(info);
        v.visit_data(info, doc, name, attr, typ, ctors)
    }
}

impl<P: Phase> Visit<P> for Codata<P> {
    fn visit<V>(&self, v: &mut V)
    where
        V: Visitor<P>,
    {
        let Codata { info, doc, name, attr, typ, dtors } = self;
        typ.visit(v);
        v.visit_info(info);
        v.visit_codata(info, doc, name, attr, typ, dtors)
    }
}

impl<P: Phase> Visit<P> for TypAbs<P> {
    fn visit<V>(&self, v: &mut V)
    where
        V: Visitor<P>,
    {
        let TypAbs { params } = self;
        v.visit_telescope(&params.params, |v, param| param.visit(v), |_| ());
        v.visit_typ_abs(params)
    }
}

impl<P: Phase> Visit<P> for Ctor<P> {
    fn visit<V>(&self, v: &mut V)
    where
        V: Visitor<P>,
    {
        let Ctor { info, doc, name, params, typ } = self;
        v.visit_telescope(&params.params, |v, param| param.visit(v), |v| typ.visit(v));
        v.visit_info(info);
        v.visit_ctor(info, doc, name, params, typ)
    }
}

impl<P: Phase> Visit<P> for Dtor<P> {
    fn visit<V>(&self, v: &mut V)
    where
        V: Visitor<P>,
    {
        let Dtor { info, doc, name, params, self_param, ret_typ } = self;
        v.visit_telescope(
            &params.params,
            |v, param| param.visit(v),
            |v| {
                v.visit_info(&self_param.info);
                self_param.typ.visit(v);
                v.visit_self_param(&self_param.info, &self_param.name, &self_param.typ, |v| {
                    ret_typ.visit(v);
                });
            },
        );
        v.visit_info(info);
        v.visit_dtor(info, doc, name, params, self_param, ret_typ)
    }
}

impl<P: Phase> Visit<P> for Def<P> {
    fn visit<V>(&self, v: &mut V)
    where
        V: Visitor<P>,
    {
        let Def { info, doc, name, attr, params, self_param, ret_typ, body } = self;
        v.visit_telescope(
            &params.params,
            |v, param| param.visit(v),
            |v| {
                v.visit_info(&self_param.info);
                self_param.typ.visit(v);
                v.visit_self_param(&self_param.info, &self_param.name, &self_param.typ, |v| {
                    ret_typ.visit(v);
                });
                body.visit(v);
            },
        );
        v.visit_info(info);
        v.visit_def(info, doc, name, attr, params, self_param, ret_typ, body)
    }
}

impl<P: Phase> Visit<P> for Codef<P> {
    fn visit<V>(&self, v: &mut V)
    where
        V: Visitor<P>,
    {
        let Codef { info, doc, name, attr, params, typ, body } = self;
        v.visit_telescope(
            &params.params,
            |v, param| param.visit(v),
            |v| {
                typ.visit(v);
                body.visit(v);
            },
        );
        v.visit_info(info);
        v.visit_codef(info, doc, name, attr, params, typ, body)
    }
}

impl<P: Phase> Visit<P> for Let<P> {
    fn visit<V>(&self, v: &mut V)
    where
        V: Visitor<P>,
    {
        let Let { info, doc, name, attr, params, typ, body } = self;
        v.visit_telescope(
            &params.params,
            |v, param| param.visit(v),
            |v| {
                typ.visit(v);
                body.visit(v);
            },
        );
        v.visit_info(info);
        v.visit_let(info, doc, name, attr, params, typ, body)
    }
}

impl<P: Phase> Visit<P> for Match<P> {
    fn visit<V>(&self, v: &mut V)
    where
        V: Visitor<P>,
    {
        let Match { info, cases, omit_absurd } = self;
        cases.visit(v);
        v.visit_info(info);
        v.visit_match(info, cases, *omit_absurd)
    }
}

impl<P: Phase> Visit<P> for Case<P> {
    fn visit<V>(&self, v: &mut V)
    where
        V: Visitor<P>,
    {
        let Case { info, name, args, body } = self;
        let TelescopeInst { params } = args;
        v.visit_telescope_inst(params, |v, arg| arg.visit(v), |v| body.visit(v));
        v.visit_info(info);
        v.visit_case(info, name, args, body)
    }
}

impl<P: Phase> Visit<P> for TypApp<P> {
    fn visit<V>(&self, v: &mut V)
    where
        V: Visitor<P>,
    {
        let TypApp { info, name, args } = self;
        args.visit(v);
        v.visit_type_info(info);
        v.visit_typ_app(info, name, args)
    }
}

impl<P: Phase> Visit<P> for Exp<P> {
    fn visit<V>(&self, v: &mut V)
    where
        V: Visitor<P>,
    {
        match self {
            Exp::Var { info, name, ctx, idx } => {
                v.visit_type_info(info);
                v.visit_idx(idx);
                v.visit_ctx(ctx);
                v.visit_exp_var(info, name, ctx, idx)
            }
            Exp::TypCtor { info, name, args } => {
                args.visit(v);
                v.visit_type_info(info);
                v.visit_exp_typ_ctor(info, name, args)
            }
            Exp::Ctor { info, name, args } => {
                args.visit(v);
                v.visit_type_info(info);
                v.visit_exp_ctor(info, name, args)
            }
            Exp::Dtor { info, exp, name, args } => {
                exp.visit(v);
                args.visit(v);
                v.visit_type_info(info);
                v.visit_exp_dtor(info, exp, name, args)
            }
            Exp::Anno { info, exp, typ } => {
                exp.visit(v);
                typ.visit(v);
                v.visit_type_info(info);
                v.visit_exp_anno(info, exp, typ)
            }
            Exp::Type { info } => {
                v.visit_type_info(info);
                v.visit_exp_type(info)
            }
            Exp::Match { info, ctx, name, on_exp, motive, ret_typ, body } => {
                v.visit_type_app_info(info);
                v.visit_ctx(ctx);
                on_exp.visit(v);
                body.visit(v);
                if let Some(m) = motive {
                    m.visit(v);
                }
                v.visit_typ(ret_typ);
                v.visit_exp_match(info, ctx, name, on_exp, ret_typ, body)
            }
            Exp::Comatch { info, ctx, name, is_lambda_sugar, body } => {
                v.visit_type_app_info(info);
                v.visit_ctx(ctx);
                body.visit(v);
                v.visit_exp_comatch(info, ctx, name, is_lambda_sugar, body)
            }
            Exp::Hole { info } => {
                v.visit_type_info(info);
                v.visit_exp_hole(info)
            }
        }
    }
}

impl<P: Phase> Visit<P> for Motive<P> {
    fn visit<V>(&self, v: &mut V)
    where
        V: Visitor<P>,
    {
        let Motive { info, param, ret_typ } = self;
        v.visit_info(info);
        param.visit(v);
        v.visit_motive_param(param, |v, param| {
            ret_typ.visit(v);
            v.visit_motive(info, param, ret_typ)
        })
    }
}

impl<P: Phase> Visit<P> for Param<P> {
    fn visit<V>(&self, v: &mut V)
    where
        V: Visitor<P>,
    {
        let Param { name, typ } = self;
        typ.visit(v);
        v.visit_param(name, typ)
    }
}

impl<P: Phase> Visit<P> for ParamInst<P> {
    fn visit<V>(&self, v: &mut V)
    where
        V: Visitor<P>,
    {
        let ParamInst { info, name, typ } = self;
        v.visit_type_info(info);
        v.visit_typ(typ);
        v.visit_param_inst(info, name, typ)
    }
}

impl<P: Phase> Visit<P> for Args<P> {
    fn visit<V>(&self, v: &mut V)
    where
        V: Visitor<P>,
    {
        let Args { args } = self;
        args.visit(v)
    }
}
