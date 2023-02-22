use std::marker::PhantomData;
use std::rc::Rc;

use data::HashMap;

use crate::common::*;

use super::def::*;
use super::source::Source;

pub fn id<P: Phase>() -> Id<P> {
    Id::default()
}

#[allow(clippy::too_many_arguments)]
#[rustfmt::skip]
pub trait Folder<P: Phase, O: Out> {
    /// Run just before a declaration is entered
    fn enter_decl(&mut self, decl: &Decl<P>) { let _ = decl; }

    fn fold_prg(&mut self, decls: O::Decls, exp: Option<O::Exp>) -> O::Prg;
    fn fold_decls(&mut self, map: HashMap<Ident, O::Decl>, order: Source) -> O::Decls;
    fn fold_decl(&mut self, decl: O::Decl) -> O::Decl;
    fn fold_decl_data(&mut self, data: O::Data) -> O::Decl;
    fn fold_decl_codata(&mut self, codata: O::Codata) -> O::Decl;
    fn fold_decl_ctor(&mut self, ctor: O::Ctor) -> O::Decl;
    fn fold_decl_dtor(&mut self, dtor: O::Dtor) -> O::Decl;
    fn fold_decl_def(&mut self, def: O::Def) -> O::Decl;
    fn fold_decl_codef(&mut self, codef: O::Codef) -> O::Decl;
    fn fold_data(&mut self, info: O::Info, doc: Option<DocComment>, name: Ident, hidden: bool, typ: O::TypAbs, ctors: Vec<Ident>) -> O::Data;
    fn fold_codata(&mut self, info: O::Info, doc: Option<DocComment>, name: Ident, hidden: bool, typ: O::TypAbs, dtors: Vec<Ident>) -> O::Codata;
    fn fold_typ_abs(&mut self, params: O::Telescope) -> O::TypAbs;
    fn fold_ctor(&mut self, info: O::Info, doc: Option<DocComment>, name: Ident, params: O::Telescope, typ: O::TypApp) -> O::Ctor;
    fn fold_dtor(&mut self, info: O::Info, doc: Option<DocComment>, name: Ident, params: O::Telescope, self_param: O::SelfParam, ret_typ: O::Exp) -> O::Dtor;
    fn fold_def(&mut self, info: O::Info, doc: Option<DocComment>, name: Ident, hidden: bool, params: O::Telescope, self_param: O::SelfParam, ret_typ: O::Exp, body: O::Match) -> O::Def;
    fn fold_codef(&mut self, info: O::Info, doc: Option<DocComment>, name: Ident, hidden: bool, params: O::Telescope, typ: O::TypApp, body: O::Comatch) -> O::Codef;
    fn fold_match(&mut self, info: O::Info, cases: Vec<O::Case>) -> O::Match;
    fn fold_comatch(&mut self, info: O::Info, cases: Vec<O::Cocase>) -> O::Comatch;
    fn fold_case(&mut self, info: O::Info, name: Ident, args: O::TelescopeInst, body: Option<O::Exp>) -> O::Case;
    fn fold_cocase(&mut self, info: O::Info, name: Ident, args: O::TelescopeInst, body: Option<O::Exp>) -> O::Cocase;
    fn fold_typ_app(&mut self, info: O::TypeInfo, name: Ident, args: Vec<O::Exp>) -> O::TypApp;
    fn fold_exp_var(&mut self, info: O::TypeInfo, name: P::VarName, idx: O::Idx) -> O::Exp;
    fn fold_exp_typ_ctor(&mut self, info: O::TypeInfo, name: Ident, args: Vec<O::Exp>) -> O::Exp;
    fn fold_exp_ctor(&mut self, info: O::TypeInfo, name: Ident, args: Vec<O::Exp>) -> O::Exp;
    fn fold_exp_dtor(&mut self, info: O::TypeInfo, exp: O::Exp, name: Ident, args: Vec<O::Exp>) -> O::Exp;
    fn fold_exp_anno(&mut self, info: O::TypeInfo, exp: O::Exp, typ: O::Exp) -> O::Exp;
    fn fold_exp_type(&mut self, info: O::TypeInfo) -> O::Exp;
    fn fold_exp_match(&mut self, info: O::TypeAppInfo, name: Ident, on_exp: O::Exp, motive: Option<O::Motive>, ret_typ: O::Typ, body: O::Match) -> O::Exp;
    fn fold_exp_comatch(&mut self, info: O::TypeAppInfo, name: Ident, body: O::Comatch) -> O::Exp;
    fn fold_exp_hole(&mut self, info: O::TypeInfo) -> O::Exp;
    fn fold_motive(&mut self, info: O::Info, param: O::ParamInst, ret_typ: O::Exp) -> O::Motive;
    // FIXME: Unifier binder handling into one method
    fn fold_motive_param<X, F>(&mut self, param: O::ParamInst, f_inner: F) -> X
    where
        F: FnOnce(&mut Self, O::ParamInst) -> X
    ;
    fn fold_telescope<X, I, F1, F2>(&mut self, params: I, f_acc: F1, f_inner: F2) -> X
    where
        I: IntoIterator<Item=Param<P>>,
        F1: Fn(&mut Self, Param<P>) -> O::Param,
        F2: FnOnce(&mut Self, O::Telescope) -> X
    ;
    fn fold_telescope_inst<X, I, F1, F2>(&mut self, params: I, f_acc: F1, f_inner: F2) -> X
    where
        I: IntoIterator<Item=ParamInst<P>>,
        F1: Fn(&mut Self, ParamInst<P>) -> O::ParamInst,
        F2: FnOnce(&mut Self, O::TelescopeInst) -> X
    ;
    fn fold_self_param<X, F>(&mut self, info: O::Info, name: Option<Ident>, typ: O::TypApp, f_inner: F) -> X
    where
        F: FnOnce(&mut Self, O::SelfParam) -> X
    ;
    fn fold_param(&mut self, name: Ident, typ: O::Exp) -> O::Param;
    fn fold_param_inst(&mut self, info: O::TypeInfo, name: Ident, typ: O::Typ) -> O::ParamInst;
    fn fold_info(&mut self, info: P::Info) -> O::Info;
    fn fold_type_info(&mut self, info: P::TypeInfo) -> O::TypeInfo;
    fn fold_type_app_info(&mut self, info: P::TypeAppInfo) -> O::TypeAppInfo;
    fn fold_idx(&mut self, idx: Idx) -> O::Idx;
    fn fold_typ(&mut self, typ: P::InfTyp) -> O::Typ;
}

pub trait Fold<P: Phase, O: Out> {
    type Out;

    fn fold<F>(self, f: &mut F) -> Self::Out
    where
        F: Folder<P, O>;
}

pub trait Out {
    type Prg;
    type Decls;
    type Decl;
    type Data;
    type Codata;
    type TypAbs;
    type Ctor;
    type Dtor;
    type Def;
    type Codef;
    type Match;
    type Comatch;
    type Case;
    type Cocase;
    type SelfParam;
    type TypApp;
    type Exp;
    type Motive;
    type Telescope;
    type TelescopeInst;
    type Param;
    type ParamInst;
    type Info;
    type TypeInfo;
    type TypeAppInfo;
    type Idx;
    type Typ;
}

#[derive(Default)]
pub struct Id<P: Phase> {
    phantom: PhantomData<P>,
}

impl<P: Phase> Out for Id<P> {
    type Prg = Prg<P>;
    type Decls = Decls<P>;
    type Decl = Decl<P>;
    type Data = Data<P>;
    type Codata = Codata<P>;
    type TypAbs = TypAbs<P>;
    type Ctor = Ctor<P>;
    type Dtor = Dtor<P>;
    type Def = Def<P>;
    type Codef = Codef<P>;
    type Match = Match<P>;
    type Comatch = Comatch<P>;
    type Case = Case<P>;
    type Cocase = Cocase<P>;
    type SelfParam = SelfParam<P>;
    type TypApp = TypApp<P>;
    type Exp = Rc<Exp<P>>;
    type Motive = Motive<P>;
    type Telescope = Telescope<P>;
    type TelescopeInst = TelescopeInst<P>;
    type Param = Param<P>;
    type ParamInst = ParamInst<P>;
    type Info = P::Info;
    type TypeInfo = P::TypeInfo;
    type TypeAppInfo = P::TypeAppInfo;
    type Idx = Idx;
    type Typ = P::InfTyp;
}

pub struct Const<T> {
    phantom: PhantomData<T>,
}

impl<T> Out for Const<T> {
    type Prg = T;
    type Decls = T;
    type Decl = T;
    type Data = T;
    type Codata = T;
    type TypAbs = T;
    type Ctor = T;
    type Dtor = T;
    type Def = T;
    type Codef = T;
    type Match = T;
    type Comatch = T;
    type Case = T;
    type Cocase = T;
    type SelfParam = T;
    type TypApp = T;
    type Exp = T;
    type Motive = T;
    type Telescope = T;
    type TelescopeInst = T;
    type Param = T;
    type ParamInst = T;
    type Info = T;
    type TypeInfo = T;
    type TypeAppInfo = T;
    type Idx = T;
    type Typ = T;
}

impl<P: Phase, O: Out, T: Fold<P, O> + Clone> Fold<P, O> for Rc<T> {
    type Out = T::Out;

    fn fold<F>(self, f: &mut F) -> Self::Out
    where
        F: Folder<P, O>,
    {
        let x = (*self).clone();
        T::fold(x, f)
    }
}

impl<P: Phase, O: Out, T: Fold<P, O>> Fold<P, O> for Option<T> {
    type Out = Option<T::Out>;

    fn fold<F>(self, f: &mut F) -> Self::Out
    where
        F: Folder<P, O>,
    {
        self.map(|inner| inner.fold(f))
    }
}

impl<P: Phase, O: Out, T: Fold<P, O>> Fold<P, O> for Vec<T> {
    type Out = Vec<T::Out>;

    fn fold<F>(self, f: &mut F) -> Self::Out
    where
        F: Folder<P, O>,
    {
        self.into_iter().map(|inner| inner.fold(f)).collect()
    }
}

impl<P: Phase, O: Out> Fold<P, O> for Prg<P>
where
    P::InfTyp: ShiftInRange,
{
    type Out = O::Prg;

    fn fold<F>(self, f: &mut F) -> Self::Out
    where
        F: Folder<P, O>,
    {
        let Prg { decls, exp } = self;
        let decls = decls.fold(f);
        let exp = exp.fold(f);
        f.fold_prg(decls, exp)
    }
}

impl<P: Phase, O: Out> Fold<P, O> for Decls<P>
where
    P::InfTyp: ShiftInRange,
{
    type Out = O::Decls;

    fn fold<F>(self, f: &mut F) -> Self::Out
    where
        F: Folder<P, O>,
    {
        let Decls { map, source } = self;
        let map = map.into_iter().map(|(name, decl)| (name, decl.fold(f))).collect();
        f.fold_decls(map, source)
    }
}

impl<P: Phase, O: Out> Fold<P, O> for Decl<P>
where
    P::InfTyp: ShiftInRange,
{
    type Out = O::Decl;

    fn fold<F>(self, f: &mut F) -> Self::Out
    where
        F: Folder<P, O>,
    {
        f.enter_decl(&self);
        let decl_out = match self {
            Decl::Data(inner) => {
                let inner = inner.fold(f);
                f.fold_decl_data(inner)
            }
            Decl::Codata(inner) => {
                let inner = inner.fold(f);
                f.fold_decl_codata(inner)
            }
            Decl::Ctor(inner) => {
                let inner = inner.fold(f);
                f.fold_decl_ctor(inner)
            }
            Decl::Dtor(inner) => {
                let inner = inner.fold(f);
                f.fold_decl_dtor(inner)
            }
            Decl::Def(inner) => {
                let inner = inner.fold(f);
                f.fold_decl_def(inner)
            }
            Decl::Codef(inner) => {
                let inner = inner.fold(f);
                f.fold_decl_codef(inner)
            }
        };
        f.fold_decl(decl_out)
    }
}

impl<P: Phase, O: Out> Fold<P, O> for Data<P> {
    type Out = O::Data;

    fn fold<F>(self, f: &mut F) -> Self::Out
    where
        F: Folder<P, O>,
    {
        let Data { info, doc, name, hidden, typ, ctors } = self;
        let typ = typ.fold(f);
        let info = f.fold_info(info);
        f.fold_data(info, doc, name, hidden, typ, ctors)
    }
}

impl<P: Phase, O: Out> Fold<P, O> for Codata<P> {
    type Out = O::Codata;

    fn fold<F>(self, f: &mut F) -> Self::Out
    where
        F: Folder<P, O>,
    {
        let Codata { info, doc, name, hidden, typ, dtors } = self;
        let typ = typ.fold(f);
        let info = f.fold_info(info);
        f.fold_codata(info, doc, name, hidden, typ, dtors)
    }
}

impl<P: Phase, O: Out> Fold<P, O> for TypAbs<P> {
    type Out = O::TypAbs;

    fn fold<F>(self, f: &mut F) -> Self::Out
    where
        F: Folder<P, O>,
    {
        let TypAbs { params } = self;
        let Telescope { params } = params;
        let params = f.fold_telescope(params, |f, param| param.fold(f), |_, params| params);
        f.fold_typ_abs(params)
    }
}

impl<P: Phase, O: Out> Fold<P, O> for Ctor<P> {
    type Out = O::Ctor;

    fn fold<F>(self, f: &mut F) -> Self::Out
    where
        F: Folder<P, O>,
    {
        let Ctor { info, doc, name, params, typ } = self;
        let Telescope { params } = params;
        let (params, typ) =
            f.fold_telescope(params, |f, param| param.fold(f), |f, params| (params, typ.fold(f)));
        let info = f.fold_info(info);
        f.fold_ctor(info, doc, name, params, typ)
    }
}

impl<P: Phase, O: Out> Fold<P, O> for Dtor<P> {
    type Out = O::Dtor;

    fn fold<F>(self, f: &mut F) -> Self::Out
    where
        F: Folder<P, O>,
    {
        let Dtor { info, doc, name, params, self_param, ret_typ } = self;
        let Telescope { params } = params;
        let (params, self_param, ret_typ) = f.fold_telescope(
            params,
            |f, param| param.fold(f),
            |f, params| {
                let self_info = f.fold_info(self_param.info);
                let self_name = self_param.name;
                let self_typ = self_param.typ.fold(f);
                f.fold_self_param(self_info, self_name, self_typ, |f, self_param| {
                    (params, self_param, ret_typ.fold(f))
                })
            },
        );
        let info = f.fold_info(info);
        f.fold_dtor(info, doc, name, params, self_param, ret_typ)
    }
}

impl<P: Phase, O: Out> Fold<P, O> for Def<P>
where
    P::InfTyp: ShiftInRange,
{
    type Out = O::Def;

    fn fold<F>(self, f: &mut F) -> Self::Out
    where
        F: Folder<P, O>,
    {
        let Def { info, doc, name, hidden, params, self_param, ret_typ, body } = self;
        let Telescope { params } = params;
        let (params, self_param, ret_typ, body) = f.fold_telescope(
            params,
            |f, param| param.fold(f),
            |f, params| {
                let self_info = f.fold_info(self_param.info);
                let self_name = self_param.name;
                let self_typ = self_param.typ.fold(f);
                let body = body.fold(f);
                f.fold_self_param(self_info, self_name, self_typ, |f, self_param| {
                    (params, self_param, ret_typ.fold(f), body)
                })
            },
        );
        let info = f.fold_info(info);
        f.fold_def(info, doc, name, hidden, params, self_param, ret_typ, body)
    }
}

impl<P: Phase, O: Out> Fold<P, O> for Codef<P> {
    type Out = O::Codef;

    fn fold<F>(self, f: &mut F) -> Self::Out
    where
        F: Folder<P, O>,
    {
        let Codef { info, doc, name, hidden, params, typ, body } = self;
        let Telescope { params } = params;
        let (params, typ, body) = f.fold_telescope(
            params,
            |f, param| param.fold(f),
            |f, params| (params, typ.fold(f), body.fold(f)),
        );
        let info = f.fold_info(info);
        f.fold_codef(info, doc, name, hidden, params, typ, body)
    }
}

impl<P: Phase, O: Out> Fold<P, O> for Match<P> {
    type Out = O::Match;

    fn fold<F>(self, f: &mut F) -> Self::Out
    where
        F: Folder<P, O>,
    {
        let Match { info, cases } = self;
        let cases = cases.fold(f);
        let info = f.fold_info(info);
        f.fold_match(info, cases)
    }
}

impl<P: Phase, O: Out> Fold<P, O> for Comatch<P> {
    type Out = O::Comatch;

    fn fold<F>(self, f: &mut F) -> Self::Out
    where
        F: Folder<P, O>,
    {
        let Comatch { info, cases } = self;
        let cases = cases.fold(f);
        let info = f.fold_info(info);
        f.fold_comatch(info, cases)
    }
}

impl<P: Phase, O: Out> Fold<P, O> for Case<P> {
    type Out = O::Case;

    fn fold<F>(self, f: &mut F) -> Self::Out
    where
        F: Folder<P, O>,
    {
        let Case { info, name, args, body } = self;
        let TelescopeInst { params } = args;
        let (args, body) =
            f.fold_telescope_inst(params, |f, arg| arg.fold(f), |f, args| (args, body.fold(f)));
        let info = f.fold_info(info);
        f.fold_case(info, name, args, body)
    }
}

impl<P: Phase, O: Out> Fold<P, O> for Cocase<P> {
    type Out = O::Cocase;

    fn fold<F>(self, f: &mut F) -> Self::Out
    where
        F: Folder<P, O>,
    {
        let Cocase { info, name, params: args, body } = self;
        let TelescopeInst { params } = args;
        let (args, body) =
            f.fold_telescope_inst(params, |f, arg| arg.fold(f), |f, args| (args, body.fold(f)));
        let info = f.fold_info(info);
        f.fold_cocase(info, name, args, body)
    }
}

impl<P: Phase, O: Out> Fold<P, O> for TypApp<P> {
    type Out = O::TypApp;

    fn fold<F>(self, f: &mut F) -> Self::Out
    where
        F: Folder<P, O>,
    {
        let TypApp { info, name, args } = self;
        let args = args.fold(f);
        let info = f.fold_type_info(info);
        f.fold_typ_app(info, name, args)
    }
}

impl<P: Phase, O: Out> Fold<P, O> for Exp<P> {
    type Out = O::Exp;

    fn fold<F>(self, f: &mut F) -> Self::Out
    where
        F: Folder<P, O>,
    {
        match self {
            Exp::Var { info, name, idx } => {
                let info = f.fold_type_info(info);
                let idx = f.fold_idx(idx);
                f.fold_exp_var(info, name, idx)
            }
            Exp::TypCtor { info, name, args } => {
                let args = args.fold(f);
                let info = f.fold_type_info(info);
                f.fold_exp_typ_ctor(info, name, args)
            }
            Exp::Ctor { info, name, args } => {
                let args = args.fold(f);
                let info = f.fold_type_info(info);
                f.fold_exp_ctor(info, name, args)
            }
            Exp::Dtor { info, exp, name, args } => {
                let exp = exp.fold(f);
                let args = args.fold(f);
                let info = f.fold_type_info(info);
                f.fold_exp_dtor(info, exp, name, args)
            }
            Exp::Anno { info, exp, typ } => {
                let exp = exp.fold(f);
                let typ = typ.fold(f);
                let info = f.fold_type_info(info);
                f.fold_exp_anno(info, exp, typ)
            }
            Exp::Type { info } => {
                let info = f.fold_type_info(info);
                f.fold_exp_type(info)
            }
            Exp::Match { info, name, on_exp, motive, ret_typ, body } => {
                let info = f.fold_type_app_info(info);
                let on_exp = on_exp.fold(f);
                let body = body.fold(f);
                let motive = motive.map(|m| m.fold(f));
                let ret_typ = f.fold_typ(ret_typ);
                f.fold_exp_match(info, name, on_exp, motive, ret_typ, body)
            }
            Exp::Comatch { info, name, body } => {
                let info = f.fold_type_app_info(info);
                let body = body.fold(f);
                f.fold_exp_comatch(info, name, body)
            }
            Exp::Hole { info } => {
                let info = f.fold_type_info(info);
                f.fold_exp_hole(info)
            }
        }
    }
}

impl<P: Phase, O: Out> Fold<P, O> for Motive<P> {
    type Out = O::Motive;

    fn fold<F>(self, f: &mut F) -> Self::Out
    where
        F: Folder<P, O>,
    {
        let Motive { info, param, ret_typ } = self;

        let info = f.fold_info(info);
        let param = param.fold(f);

        f.fold_motive_param(param, |f, param| {
            let ret_typ = ret_typ.fold(f);
            f.fold_motive(info, param, ret_typ)
        })
    }
}

impl<P: Phase, O: Out> Fold<P, O> for Param<P> {
    type Out = O::Param;

    fn fold<F>(self, f: &mut F) -> Self::Out
    where
        F: Folder<P, O>,
    {
        let Param { name, typ } = self;
        let typ = typ.fold(f);
        f.fold_param(name, typ)
    }
}

impl<P: Phase, O: Out> Fold<P, O> for ParamInst<P> {
    type Out = O::ParamInst;

    fn fold<F>(self, f: &mut F) -> Self::Out
    where
        F: Folder<P, O>,
    {
        let ParamInst { info, name, typ } = self;
        let info = f.fold_type_info(info);
        let typ = f.fold_typ(typ);
        f.fold_param_inst(info, name, typ)
    }
}
