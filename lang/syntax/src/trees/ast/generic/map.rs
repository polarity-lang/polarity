use std::rc::Rc;

use data::HashMap;

use crate::common::*;

use super::def::*;
use super::fold::*;
use super::source::Source;

#[rustfmt::skip]
pub trait Mapper<P: Phase> {
    /// Run just before a declaration is entered
    fn enter_decl(&mut self, decl: &Decl<P>) { let _ = decl; }

    fn map_prg(&mut self, decls: Decls<P>, exp: Option<Rc<Exp<P>>>) -> Prg<P> {
        Prg { decls, exp }
    }
    fn map_decls(&mut self, map: HashMap<Ident, Decl<P>>, source: Source) -> Decls<P> {
        Decls { map, source }
    }
    fn map_decl(&mut self, decl: Decl<P>) -> Decl<P> {
        decl
    }
    fn map_decl_data(&mut self, data: Data<P>) -> Decl<P> {
        Decl::Data(data)
    }
    fn map_decl_codata(&mut self, codata: Codata<P>) -> Decl<P> {
        Decl::Codata(codata)
    }
    fn map_decl_ctor(&mut self, ctor: Ctor<P>) -> Decl<P> {
        Decl::Ctor(ctor)
    }
    fn map_decl_dtor(&mut self, dtor: Dtor<P>) -> Decl<P> {
        Decl::Dtor(dtor)
    }
    fn map_decl_def(&mut self, def: Def<P>) -> Decl<P> {
        Decl::Def(def)
    }
    fn map_decl_codef(&mut self, codef: Codef<P>) -> Decl<P> {
        Decl::Codef(codef)
    }
    fn map_data(&mut self, info: P::Info, name: Ident, typ: Rc<TypAbs<P>>, ctors: Vec<Ident>, impl_block: Option<Impl<P>>) -> Data<P> {
        Data { info, name, typ, ctors, impl_block }
    }
    fn map_codata(&mut self, info: P::Info, name: Ident, typ: Rc<TypAbs<P>>, dtors: Vec<Ident>, impl_block: Option<Impl<P>>) -> Codata<P> {
        Codata { info, name, typ, dtors, impl_block }
    }
    fn map_impl(&mut self, info: P::Info, name: Ident, defs: Vec<Ident>) -> Impl<P> {
        Impl { info, name, defs }
    }
    fn map_typ_abs(&mut self, params: Telescope<P>) -> TypAbs<P> {
        TypAbs { params }
    }
    fn map_ctor(&mut self, info: P::Info, name: Ident, params: Telescope<P>, typ: TypApp<P>) -> Ctor<P> {
        Ctor { info, name, params, typ }
    }
    fn map_dtor(&mut self, info: P::Info, name: Ident, params: Telescope<P>, on_typ: TypApp<P>, in_typ: Rc<Exp<P>>) -> Dtor<P> {
        Dtor { info, name, params, on_typ, in_typ }
    }
    fn map_def(&mut self, info: P::Info, name: Ident, params: Telescope<P>, on_typ: TypApp<P>, in_typ: Rc<Exp<P>>, body: Match<P>) -> Def<P> {
        Def { info, name, params, on_typ, in_typ, body }
    }
    fn map_codef(&mut self, info: P::Info, name: Ident, params: Telescope<P>, typ: TypApp<P>, body: Comatch<P>) -> Codef<P> {
        Codef { info, name, params, typ, body }
    }
    fn map_match(&mut self, info: P::Info, cases: Vec<Case<P>>) -> Match<P> {
        Match { info, cases }
    }
    fn map_comatch(&mut self, info: P::Info, cases: Vec<Cocase<P>>) -> Comatch<P> {
        Comatch { info, cases }
    }
    fn map_case(&mut self, info: P::Info, name: Ident, args: TelescopeInst<P>, body: Option<Rc<Exp<P>>>) -> Case<P> {
        Case { info, name, args, body }
    }
    fn map_cocase(&mut self, info: P::Info, name: Ident, args: TelescopeInst<P>, body: Option<Rc<Exp<P>>>) -> Cocase<P> {
        Cocase { info, name, args, body }
    }
    fn map_typ_app(&mut self, info: P::TypeInfo, name: Ident, args: Vec<Rc<Exp<P>>>) -> TypApp<P> {
        TypApp { info, name, args }
    }
    fn map_exp_var(&mut self, info: P::TypeInfo, name: P::VarName, idx: Idx) -> Exp<P> {
        Exp::Var { info, name, idx }
    }
    fn map_exp_typ_ctor(&mut self, info: P::TypeInfo, name: Ident, args: Vec<Rc<Exp<P>>>) -> Exp<P> {
        Exp::TypCtor { info, name, args }
    }
    fn map_exp_ctor(&mut self, info: P::TypeInfo, name: Ident, args: Vec<Rc<Exp<P>>>) -> Exp<P> {
        Exp::Ctor { info, name, args }
    }
    fn map_exp_dtor(&mut self, info: P::TypeInfo, exp: Rc<Exp<P>>, name: Ident, args: Vec<Rc<Exp<P>>>) -> Exp<P> {
        Exp::Dtor { info, exp, name, args }
    }
    fn map_exp_anno(&mut self, info: P::TypeInfo, exp: Rc<Exp<P>>, typ: Rc<Exp<P>>) -> Exp<P> {
        Exp::Anno { info, exp, typ }
    }
    fn map_exp_type(&mut self, info: P::TypeInfo) -> Exp<P> {
        Exp::Type { info }
    }
    fn map_exp_match(&mut self, info: P::TypeAppInfo, name: Ident, on_exp: Rc<Exp<P>>, in_typ: P::Typ, body: Match<P>) -> Exp<P> {
        Exp::Match { info, name, on_exp, in_typ, body }
    }
    fn map_exp_comatch(&mut self, info: P::TypeAppInfo, name: Ident, body: Comatch<P>) -> Exp<P> {
        Exp::Comatch { info, name, body }
    }
    fn map_telescope<X, I, F1, F2>(&mut self, params: I, f_acc: F1, f_inner: F2) -> X
    where
        I: IntoIterator<Item=Param<P>>,
        F1: Fn(&mut Self, Param<P>) -> Param<P>,
        F2: FnOnce(&mut Self, Telescope<P>) -> X
    {
        let params = params.into_iter().map(|param| f_acc(self, param)).collect();
        let params = Telescope { params };
        f_inner(self, params)
    }
    fn map_telescope_inst<X, I, F1, F2>(&mut self, params: I, f_acc: F1, f_inner: F2) -> X
    where
        I: IntoIterator<Item=ParamInst<P>>,
        F1: Fn(&mut Self, ParamInst<P>) -> ParamInst<P>,
        F2: FnOnce(&mut Self, TelescopeInst<P>) -> X
    {
        let params = params.into_iter().map(|param| f_acc(self, param)).collect();
        let params = TelescopeInst { params };
        f_inner(self, params)
    }
    fn map_param(&mut self, name: Ident, typ: Rc<Exp<P>>) -> Param<P> {
        Param { name, typ }
    }
    fn map_param_inst(&mut self, info: P::TypeInfo, name: Ident, typ: P::Typ) -> ParamInst<P> {
        ParamInst { info, name, typ }
    }
    fn map_info(&mut self, info: P::Info) -> P::Info {
        info
    }
    fn map_type_info(&mut self, info: P::TypeInfo) -> P::TypeInfo {
        info
    }
    fn map_type_app_info(&mut self, info: P::TypeAppInfo) -> P::TypeAppInfo {
        info
    }
    fn map_idx(&mut self, idx: Idx) -> Idx {
        idx
    }
}

impl<P: Phase> Mapper<P> for Id<P> {}

pub trait Map<P: Phase> {
    fn map<M>(self, m: &mut M) -> Self
    where
        M: Mapper<P>;
}

impl<P: Phase, T: Fold<P, Id<P>, Out = Self>> Map<P> for T {
    fn map<M>(self, m: &mut M) -> Self
    where
        M: Mapper<P>,
    {
        self.fold(m)
    }
}

#[rustfmt::skip]
impl<P: Phase, T: Mapper<P>> Folder<P, Id<P>> for T {
    fn enter_decl(&mut self, decl: &Decl<P>) {
        self.enter_decl(decl)
    }

    fn fold_prg(&mut self, decls: <Id<P> as Out>::Decls, exp: Option<<Id<P> as Out>::Exp>) -> <Id<P> as Out>::Prg {
        self.map_prg(decls, exp)
    }

    fn fold_decls(&mut self, map: HashMap<Ident, <Id<P> as Out>::Decl>, source: Source) -> <Id<P> as Out>::Decls {
        self.map_decls(map, source)
    }

    fn fold_decl(&mut self, decl: <Id<P> as Out>::Decl) -> <Id<P> as Out>::Decl {
        self.map_decl(decl)
    }

    fn fold_decl_data(&mut self, data: <Id<P> as Out>::Data) -> <Id<P> as Out>::Decl {
        self.map_decl_data(data)
    }

    fn fold_decl_codata(&mut self, codata: <Id<P> as Out>::Codata) -> <Id<P> as Out>::Decl {
        self.map_decl_codata(codata)
    }

    fn fold_decl_ctor(&mut self, ctor: <Id<P> as Out>::Ctor) -> <Id<P> as Out>::Decl {
        self.map_decl_ctor(ctor)
    }

    fn fold_decl_dtor(&mut self, dtor: <Id<P> as Out>::Dtor) -> <Id<P> as Out>::Decl {
        self.map_decl_dtor(dtor)
    }

    fn fold_decl_def(&mut self, def: <Id<P> as Out>::Def) -> <Id<P> as Out>::Decl {
        self.map_decl_def(def)
    }

    fn fold_decl_codef(&mut self, codef: <Id<P> as Out>::Codef) -> <Id<P> as Out>::Decl {
        self.map_decl_codef(codef)
    }

    fn fold_data(&mut self, info: <Id<P> as Out>::Info, name: Ident, typ: <Id<P> as Out>::TypAbs, ctors: Vec<Ident>, impl_block: Option<<Id<P> as Out>::Impl>) -> <Id<P> as Out>::Data {
        self.map_data(info, name, Rc::new(typ), ctors, impl_block)
    }

    fn fold_codata(&mut self, info: <Id<P> as Out>::Info, name: Ident, typ: <Id<P> as Out>::TypAbs, dtors: Vec<Ident>, impl_block: Option<<Id<P> as Out>::Impl>) -> <Id<P> as Out>::Codata {
        self.map_codata(info, name, Rc::new(typ), dtors, impl_block)
    }

    fn fold_impl(&mut self, info: <Id<P> as Out>::Info, name: Ident, defs: Vec<Ident>) -> <Id<P> as Out>::Impl {
        self.map_impl(info, name, defs)
    }

    fn fold_typ_abs(&mut self, params: <Id<P> as Out>::Telescope) -> <Id<P> as Out>::TypAbs {
        self.map_typ_abs(params)
    }

    fn fold_ctor(&mut self, info: <Id<P> as Out>::Info, name: Ident, params: <Id<P> as Out>::Telescope, typ: <Id<P> as Out>::TypApp) -> <Id<P> as Out>::Ctor {
        self.map_ctor(info, name, params, typ)
    }

    fn fold_dtor(&mut self, info: <Id<P> as Out>::Info, name: Ident, params: <Id<P> as Out>::Telescope, on_typ: <Id<P> as Out>::TypApp, in_typ: <Id<P> as Out>::Exp) -> <Id<P> as Out>::Dtor {
        self.map_dtor(info, name, params, on_typ, in_typ)
    }

    fn fold_def(&mut self, info: <Id<P> as Out>::Info, name: Ident, params: <Id<P> as Out>::Telescope, on_typ: <Id<P> as Out>::TypApp, in_typ: <Id<P> as Out>::Exp, body: <Id<P> as Out>::Match) -> <Id<P> as Out>::Def {
        self.map_def(info, name, params, on_typ, in_typ, body)
    }

    fn fold_codef(&mut self, info: <Id<P> as Out>::Info, name: Ident, params: <Id<P> as Out>::Telescope, typ: <Id<P> as Out>::TypApp, body: <Id<P> as Out>::Comatch) -> <Id<P> as Out>::Codef {
        self.map_codef(info, name, params, typ, body)
    }

    fn fold_match(&mut self, info: <Id<P> as Out>::Info, cases: Vec<<Id<P> as Out>::Case>) -> <Id<P> as Out>::Match {
        self.map_match(info, cases)
    }

    fn fold_comatch(&mut self, info: <Id<P> as Out>::Info, cases: Vec<<Id<P> as Out>::Cocase>) -> <Id<P> as Out>::Comatch {
        self.map_comatch(info, cases)
    }

    fn fold_case(&mut self, info: <Id<P> as Out>::Info, name: Ident, args: <Id<P> as Out>::TelescopeInst, body: Option<<Id<P> as Out>::Exp>) -> <Id<P> as Out>::Case {
        self.map_case(info, name, args, body)
    }

    fn fold_cocase(&mut self, info: <Id<P> as Out>::Info, name: Ident, args: <Id<P> as Out>::TelescopeInst, body: Option<<Id<P> as Out>::Exp>) -> <Id<P> as Out>::Cocase {
        self.map_cocase(info, name, args, body)
    }

    fn fold_typ_app(&mut self, info: <Id<P> as Out>::TypeInfo, name: Ident, args: Vec<<Id<P> as Out>::Exp>) -> <Id<P> as Out>::TypApp {
        self.map_typ_app(info, name, args)
    }

    fn fold_exp_var(&mut self, info: <Id<P> as Out>::TypeInfo, name: <P as Phase>::VarName, idx: <Id<P> as Out>::Idx) -> <Id<P> as Out>::Exp {
        Rc::new(self.map_exp_var(info, name, idx))
    }

    fn fold_exp_typ_ctor(&mut self, info: <Id<P> as Out>::TypeInfo, name: Ident, args: Vec<<Id<P> as Out>::Exp>) -> <Id<P> as Out>::Exp {
        Rc::new(self.map_exp_typ_ctor(info, name, args))
    }

    fn fold_exp_ctor(&mut self, info: <Id<P> as Out>::TypeInfo, name: Ident, args: Vec<<Id<P> as Out>::Exp>) -> <Id<P> as Out>::Exp {
        Rc::new(self.map_exp_ctor(info, name, args))
    }

    fn fold_exp_dtor(&mut self, info: <Id<P> as Out>::TypeInfo, exp: <Id<P> as Out>::Exp, name: Ident, args: Vec<<Id<P> as Out>::Exp>) -> <Id<P> as Out>::Exp {
        Rc::new(self.map_exp_dtor(info, exp, name, args))
    }

    fn fold_exp_anno(&mut self, info: <Id<P> as Out>::TypeInfo, exp: <Id<P> as Out>::Exp, typ: <Id<P> as Out>::Exp) -> <Id<P> as Out>::Exp {
        Rc::new(self.map_exp_anno(info, exp, typ))
    }

    fn fold_exp_type(&mut self, info: <Id<P> as Out>::TypeInfo) -> <Id<P> as Out>::Exp {
        Rc::new(self.map_exp_type(info))
    }

    fn fold_exp_match(&mut self, info: <Id<P> as Out>::TypeAppInfo, name: Ident, on_exp: <Id<P> as Out>::Exp, in_typ: <Id<P> as Out>::Typ, body: <Id<P> as Out>::Match) -> <Id<P> as Out>::Exp {
        Rc::new(self.map_exp_match(info, name, on_exp, in_typ, body))
    }

    fn fold_exp_comatch(&mut self, info: <Id<P> as Out>::TypeAppInfo, name: Ident, body: <Id<P> as Out>::Comatch) -> <Id<P> as Out>::Exp {
        Rc::new(self.map_exp_comatch(info, name, body))
    }

    fn fold_telescope<X, I, F1, F2>(&mut self, params: I, f_acc: F1, f_inner: F2) -> X
    where
        I: IntoIterator<Item=Param<P>>,
        F1: Fn(&mut Self, Param<P>) -> <Id<P> as Out>::Param,
        F2: FnOnce(&mut Self, <Id<P> as Out>::Telescope) -> X
    {
        self.map_telescope(params,
            |mapper, param| f_acc(mapper, param),
            |mapper, params| f_inner(mapper, params)
        )
    }

    fn fold_telescope_inst<X, I, F1, F2>(&mut self, params: I, f_acc: F1, f_inner: F2) -> X
        where
            I: IntoIterator<Item=ParamInst<P>>,
            F1: Fn(&mut Self, ParamInst<P>) -> <Id<P> as Out>::ParamInst,
            F2: FnOnce(&mut Self, <Id<P> as Out>::TelescopeInst) -> X
    {
        self.map_telescope_inst(params,
            |mapper, param| f_acc(mapper, param),
            |mapper, params| f_inner(mapper, params)
        )
    }

    fn fold_param(&mut self, name: Ident, typ: <Id<P> as Out>::Exp) -> <Id<P> as Out>::Param {
        self.map_param(name, typ)
    }

    fn fold_param_inst(&mut self, info: <Id<P> as Out>::TypeInfo, name: Ident, typ: <Id<P> as Out>::Typ) -> <Id<P> as Out>::ParamInst {
        self.map_param_inst(info, name, typ)
    }

    fn fold_info(&mut self, info: <P as Phase>::Info) -> <Id<P> as Out>::Info {
        self.map_info(info)
    }

    fn fold_type_info(&mut self, info: <P as Phase>::TypeInfo) -> <Id<P> as Out>::TypeInfo {
        self.map_type_info(info)
    }

    fn fold_type_app_info(&mut self, info: <P as Phase>::TypeAppInfo) -> <Id<P> as Out>::TypeAppInfo {
        self.map_type_app_info(info)
    }

    fn fold_idx(&mut self, idx: Idx) -> <Id<P> as Out>::Idx {
        self.map_idx(idx)
    }

    fn fold_typ(&mut self, typ: <P as Phase>::Typ) -> <Id<P> as Out>::Typ {
        typ
    }
}
