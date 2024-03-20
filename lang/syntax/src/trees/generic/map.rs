use std::rc::Rc;

use codespan::Span;

use crate::common::*;

use super::def::*;
use super::fold::*;
use super::lookup_table::LookupTable;

#[allow(clippy::too_many_arguments)]
#[rustfmt::skip]
pub trait Mapper<P: Phase> {
    /// Run just before a declaration is entered
    fn enter_decl(&mut self, decl: &Decl<P>) { let _ = decl; }

    fn map_prg(&mut self, decls: Decls<P>, exp: Option<Rc<Exp<P>>>) -> Prg<P> {
        Prg { decls, exp }
    }
    fn map_decls(&mut self, map: HashMap<Ident, Decl<P>>, lookup_table: LookupTable) -> Decls<P> {
        Decls { map, lookup_table }
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
    fn map_data(&mut self, info: Option<Span>, doc: Option<DocComment>, name: Ident, attr: Attribute, typ: Rc<TypAbs<P>>, ctors: Vec<Ident>) -> Data<P> {
        Data { info, doc, name, attr, typ, ctors }
    }
    fn map_codata(&mut self, info: Option<Span>, doc: Option<DocComment>, name: Ident, attr: Attribute, typ: Rc<TypAbs<P>>, dtors: Vec<Ident>) -> Codata<P> {
        Codata { info, doc, name, attr, typ, dtors }
    }
    fn map_typ_abs(&mut self, params: Telescope<P>) -> TypAbs<P> {
        TypAbs { params }
    }
    fn map_ctor(&mut self, info: Option<Span>, doc: Option<DocComment>, name: Ident, params: Telescope<P>, typ: TypApp<P>) -> Ctor<P> {
        Ctor { info, doc, name, params, typ }
    }
    fn map_dtor(&mut self, info: Option<Span>, doc: Option<DocComment>, name: Ident, params: Telescope<P>, self_param: SelfParam<P>, ret_typ: Rc<Exp<P>>) -> Dtor<P> {
        Dtor { info, doc, name, params, self_param, ret_typ }
    }
    fn map_def(&mut self, info: Option<Span>, doc: Option<DocComment>, name: Ident, attr: Attribute, params: Telescope<P>, self_param: SelfParam<P>, ret_typ: Rc<Exp<P>>, body: Match<P>) -> Def<P> {
        Def { info, doc, name, attr, params, self_param, ret_typ, body }
    }
    fn map_codef(&mut self, info: Option<Span>, doc: Option<DocComment>, name: Ident, attr: Attribute, params: Telescope<P>, typ: TypApp<P>, body: Match<P>) -> Codef<P> {
        Codef { info, doc, name, attr, params, typ, body }
    }
    fn map_match(&mut self, info: Option<Span>, cases: Vec<Case<P>>, omit_absurd: bool) -> Match<P> {
        Match { info, cases, omit_absurd }
    }
    fn map_case(&mut self, info: Option<Span>, name: Ident, args: TelescopeInst<P>, body: Option<Rc<Exp<P>>>) -> Case<P> {
        Case { info, name, args, body }
    }
    fn map_typ_app(&mut self, info: P::TypeInfo, name: Ident, args: Args<P>) -> TypApp<P> {
        TypApp { info, name, args }
    }
    fn map_exp_var(&mut self, info: P::TypeInfo, name: Ident, ctx: P::Ctx, idx: Idx) -> Exp<P> {
        Exp::Var { info, name, ctx, idx }
    }
    fn map_exp_typ_ctor(&mut self, info: P::TypeInfo, name: Ident, args: Args<P>) -> Exp<P> {
        Exp::TypCtor { info, name, args }
    }
    fn map_exp_ctor(&mut self, info: P::TypeInfo, name: Ident, args: Args<P>) -> Exp<P> {
        Exp::Ctor { info, name, args }
    }
    fn map_exp_dtor(&mut self, info: P::TypeInfo, exp: Rc<Exp<P>>, name: Ident, args: Args<P>) -> Exp<P> {
        Exp::Dtor { info, exp, name, args }
    }
    fn map_exp_anno(&mut self, info: P::TypeInfo, exp: Rc<Exp<P>>, typ: Rc<Exp<P>>) -> Exp<P> {
        Exp::Anno { info, exp, typ }
    }
    fn map_exp_type(&mut self, info: P::TypeInfo) -> Exp<P> {
        Exp::Type { info }
    }
    fn map_exp_match(&mut self, info: P::TypeAppInfo, ctx: P::Ctx, name: Label, on_exp: Rc<Exp<P>>, motive: Option<Motive<P>>, ret_typ: P::InfTyp, body: Match<P>) -> Exp<P> {
        Exp::Match { info, ctx, name, on_exp, motive, ret_typ, body }
    }
    fn map_exp_comatch(&mut self, info: P::TypeAppInfo, ctx: P::Ctx, name: Label, is_lambda_sugar: bool, body: Match<P>) -> Exp<P> {
        Exp::Comatch { info, ctx, name, is_lambda_sugar, body }
    }
    fn map_exp_hole(&mut self, info: P::TypeInfo) -> Exp<P> {
        Exp::Hole { info }
    }
    fn map_motive(&mut self, info: Option<Span>, param: ParamInst<P>, ret_typ: Rc<Exp<P>>) -> Motive<P> {
        Motive { info, param, ret_typ }
    }
    fn map_motive_param<X, F>(&mut self, param: ParamInst<P>, f_inner: F) -> X
    where
        F: FnOnce(&mut Self, ParamInst<P>) -> X
    {
        f_inner(self, param)
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
    fn map_self_param<X, F>(&mut self, info: Option<Span>, name: Option<Ident>, typ: TypApp<P>, f_inner: F) -> X
    where
        F: FnOnce(&mut Self, SelfParam<P>) -> X
    {
        f_inner(self, SelfParam { info, name, typ })
    }
    fn map_args(&mut self, args: Vec<Rc<Exp<P>>>) -> Args<P> {
        Args { args }
    }
    fn map_param(&mut self, name: Ident, typ: Rc<Exp<P>>) -> Param<P> {
        Param { name, typ }
    }
    fn map_param_inst(&mut self, info: P::TypeInfo, name: Ident, typ: P::InfTyp) -> ParamInst<P> {
        ParamInst { info, name, typ }
    }
    fn map_info(&mut self, info: Option<Span>) -> Option<Span> {
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
    fn map_ctx(&mut self, ctx: P::Ctx) -> P::Ctx {
        ctx
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

    fn fold_decls(&mut self, map: HashMap<Ident, <Id<P> as Out>::Decl>, source: LookupTable) -> <Id<P> as Out>::Decls {
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

    fn fold_data(&mut self, info: Option<Span>, doc: Option<DocComment>, name: Ident, attr: Attribute, typ: <Id<P> as Out>::TypAbs, ctors: Vec<Ident>) -> <Id<P> as Out>::Data {
        self.map_data(info, doc, name, attr, Rc::new(typ), ctors)
    }

    fn fold_codata(&mut self, info: Option<Span>, doc: Option<DocComment>, name: Ident, attr: Attribute, typ: <Id<P> as Out>::TypAbs, dtors: Vec<Ident>) -> <Id<P> as Out>::Codata {
        self.map_codata(info, doc, name, attr, Rc::new(typ), dtors)
    }

    fn fold_typ_abs(&mut self, params: <Id<P> as Out>::Telescope) -> <Id<P> as Out>::TypAbs {
        self.map_typ_abs(params)
    }

    fn fold_ctor(&mut self, info: Option<Span>, doc: Option<DocComment>, name: Ident, params: <Id<P> as Out>::Telescope, typ: <Id<P> as Out>::TypApp) -> <Id<P> as Out>::Ctor {
        self.map_ctor(info, doc, name, params, typ)
    }

    fn fold_dtor(&mut self, info: Option<Span>, doc: Option<DocComment>, name: Ident, params: <Id<P> as Out>::Telescope, self_param: <Id<P> as Out>::SelfParam, ret_typ: <Id<P> as Out>::Exp) -> <Id<P> as Out>::Dtor {
        self.map_dtor(info, doc, name, params, self_param, ret_typ)
    }

    fn fold_def(&mut self, info: Option<Span>, doc: Option<DocComment>, name: Ident, attr: Attribute, params: <Id<P> as Out>::Telescope, self_param: <Id<P> as Out>::SelfParam, ret_typ: <Id<P> as Out>::Exp, body: <Id<P> as Out>::Match) -> <Id<P> as Out>::Def {
        self.map_def(info, doc, name, attr, params, self_param, ret_typ, body)
    }

    fn fold_codef(&mut self, info: Option<Span>, doc: Option<DocComment>, name: Ident, attr: Attribute, params: <Id<P> as Out>::Telescope, typ: <Id<P> as Out>::TypApp, body: <Id<P> as Out>::Match) -> <Id<P> as Out>::Codef {
        self.map_codef(info, doc, name, attr, params, typ, body)
    }

    fn fold_match(&mut self, info: Option<Span>, cases: Vec<<Id<P> as Out>::Case>, omit_absurd: bool) -> <Id<P> as Out>::Match {
        self.map_match(info, cases, omit_absurd)
    }

    fn fold_case(&mut self, info: Option<Span>, name: Ident, args: <Id<P> as Out>::TelescopeInst, body: Option<<Id<P> as Out>::Exp>) -> <Id<P> as Out>::Case {
        self.map_case(info, name, args, body)
    }

    fn fold_typ_app(&mut self, info: <Id<P> as Out>::TypeInfo, name: Ident, args: <Id<P> as Out>::Args) -> <Id<P> as Out>::TypApp {
        self.map_typ_app(info, name, args)
    }

    fn fold_exp_var(&mut self, info: <Id<P> as Out>::TypeInfo, name: Ident, ctx: <P as Phase>::Ctx, idx: <Id<P> as Out>::Idx) -> <Id<P> as Out>::Exp {
        Rc::new(self.map_exp_var(info, name, ctx, idx))
    }

    fn fold_exp_typ_ctor(&mut self, info: <Id<P> as Out>::TypeInfo, name: Ident, args: <Id<P> as Out>::Args) -> <Id<P> as Out>::Exp {
        Rc::new(self.map_exp_typ_ctor(info, name, args))
    }

    fn fold_exp_ctor(&mut self, info: <Id<P> as Out>::TypeInfo, name: Ident, args: <Id<P> as Out>::Args) -> <Id<P> as Out>::Exp {
        Rc::new(self.map_exp_ctor(info, name, args))
    }

    fn fold_exp_dtor(&mut self, info: <Id<P> as Out>::TypeInfo, exp: <Id<P> as Out>::Exp, name: Ident, args: <Id<P> as Out>::Args) -> <Id<P> as Out>::Exp {
        Rc::new(self.map_exp_dtor(info, exp, name, args))
    }

    fn fold_exp_anno(&mut self, info: <Id<P> as Out>::TypeInfo, exp: <Id<P> as Out>::Exp, typ: <Id<P> as Out>::Exp) -> <Id<P> as Out>::Exp {
        Rc::new(self.map_exp_anno(info, exp, typ))
    }

    fn fold_exp_type(&mut self, info: <Id<P> as Out>::TypeInfo) -> <Id<P> as Out>::Exp {
        Rc::new(self.map_exp_type(info))
    }

    fn fold_exp_match(&mut self, info: <Id<P> as Out>::TypeAppInfo, ctx: <Id<P> as Out>::Ctx, name: Label, on_exp: <Id<P> as Out>::Exp, motive: Option<<Id<P> as Out>::Motive>, ret_typ: <Id<P> as Out>::Typ, body: <Id<P> as Out>::Match) -> <Id<P> as Out>::Exp {
        Rc::new(self.map_exp_match(info, ctx, name, on_exp, motive, ret_typ, body))
    }

    fn fold_exp_comatch(&mut self, info: <Id<P> as Out>::TypeAppInfo, ctx: <Id<P> as Out>::Ctx, name: Label, is_lambda_sugar: bool, body: <Id<P> as Out>::Match) -> <Id<P> as Out>::Exp {
        Rc::new(self.map_exp_comatch(info, ctx, name, is_lambda_sugar, body))
    }

    fn fold_exp_hole(&mut self, info: <Id<P> as Out>::TypeInfo) -> <Id<P> as Out>::Exp {
        Rc::new(self.map_exp_hole(info))
    }

    fn fold_motive(&mut self, info: Option<Span>, param: <Id<P> as Out>::ParamInst, ret_typ: <Id<P> as Out>::Exp) -> <Id<P> as Out>::Motive {
        self.map_motive(info, param, ret_typ)
    }

    fn fold_motive_param<X, F>(&mut self, param: <Id<P> as Out>::ParamInst, f_inner: F) -> X
        where
            F: FnOnce(&mut Self, <Id<P> as Out>::ParamInst) -> X
    {
        self.map_motive_param(param, f_inner)
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

    fn fold_self_param<X, F>(&mut self, info: Option<Span>, name: Option<Ident>, typ: <Id<P> as Out>::TypApp, f_inner: F) -> X
    where
        F: FnOnce(&mut Self, <Id<P> as Out>::SelfParam) -> X
    {
        self.map_self_param(info, name, typ, f_inner)
    }

    fn fold_param(&mut self, name: Ident, typ: <Id<P> as Out>::Exp) -> <Id<P> as Out>::Param {
        self.map_param(name, typ)
    }

    fn fold_param_inst(&mut self, info: <Id<P> as Out>::TypeInfo, name: Ident, typ: <Id<P> as Out>::Typ) -> <Id<P> as Out>::ParamInst {
        self.map_param_inst(info, name, typ)
    }

    fn fold_args(&mut self, args: Vec<<Id<P> as Out>::Exp>) -> <Id<P> as Out>::Args {
        self.map_args(args)
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

    fn fold_typ(&mut self, typ: <P as Phase>::InfTyp) -> <Id<P> as Out>::Typ {
        typ
    }

    fn fold_ctx(&mut self, ctx: <P as Phase>::Ctx) -> <Id<P> as Out>::Ctx {
        self.map_ctx(ctx)
    }
}
