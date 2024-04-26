//! Convert a typed syntax tree to a representation suitable for program transformations

use std::rc::Rc;

use super::def::*;
use crate::ust;

pub trait ForgetTST {
    type Target;

    fn forget_tst(&self) -> Self::Target;
}

impl<T: ForgetTST> ForgetTST for Rc<T> {
    type Target = Rc<T::Target>;

    fn forget_tst(&self) -> Self::Target {
        Rc::new(T::forget_tst(self))
    }
}

impl<T: ForgetTST> ForgetTST for Option<T> {
    type Target = Option<T::Target>;

    fn forget_tst(&self) -> Self::Target {
        self.as_ref().map(ForgetTST::forget_tst)
    }
}

impl<T: ForgetTST> ForgetTST for Vec<T> {
    type Target = Vec<T::Target>;

    fn forget_tst(&self) -> Self::Target {
        self.iter().map(ForgetTST::forget_tst).collect()
    }
}

impl ForgetTST for Prg {
    type Target = ust::Prg;

    fn forget_tst(&self) -> Self::Target {
        let Prg { decls } = self;

        ust::Prg { decls: decls.forget_tst() }
    }
}

impl ForgetTST for Decls {
    type Target = ust::Decls;

    fn forget_tst(&self) -> Self::Target {
        let Decls { map, lookup_table } = self;

        ust::Decls {
            map: map.iter().map(|(name, decl)| (name.clone(), decl.forget_tst())).collect(),
            lookup_table: lookup_table.clone(),
        }
    }
}

impl ForgetTST for Decl {
    type Target = ust::Decl;

    fn forget_tst(&self) -> Self::Target {
        match self {
            Decl::Data(data) => ust::Decl::Data(data.forget_tst()),
            Decl::Codata(codata) => ust::Decl::Codata(codata.forget_tst()),
            Decl::Ctor(ctor) => ust::Decl::Ctor(ctor.forget_tst()),
            Decl::Dtor(dtor) => ust::Decl::Dtor(dtor.forget_tst()),
            Decl::Def(def) => ust::Decl::Def(def.forget_tst()),
            Decl::Codef(codef) => ust::Decl::Codef(codef.forget_tst()),
            Decl::Let(tl_let) => ust::Decl::Let(tl_let.forget_tst()),
        }
    }
}

impl ForgetTST for Data {
    type Target = ust::Data;

    fn forget_tst(&self) -> Self::Target {
        let Data { span, doc, name, attr, typ, ctors } = self;

        ust::Data {
            span: *span,
            name: name.clone(),
            doc: doc.clone(),
            attr: attr.clone(),
            typ: typ.forget_tst(),
            ctors: ctors.clone(),
        }
    }
}

impl ForgetTST for Let {
    type Target = ust::Let;

    fn forget_tst(&self) -> Self::Target {
        let Let { span, doc, name, attr, params, typ, body } = self;

        ust::Let {
            span: *span,
            doc: doc.clone(),
            name: name.clone(),
            attr: attr.clone(),
            params: params.forget_tst(),
            typ: typ.forget_tst(),
            body: body.forget_tst(),
        }
    }
}

impl ForgetTST for Codata {
    type Target = ust::Codata;

    fn forget_tst(&self) -> Self::Target {
        let Codata { span, doc, name, attr, typ, dtors } = self;

        ust::Codata {
            span: *span,
            doc: doc.clone(),
            name: name.clone(),
            attr: attr.clone(),
            typ: typ.forget_tst(),
            dtors: dtors.clone(),
        }
    }
}

impl ForgetTST for TypAbs {
    type Target = ust::TypAbs;

    fn forget_tst(&self) -> Self::Target {
        let TypAbs { params } = self;

        ust::TypAbs { params: params.forget_tst() }
    }
}

impl ForgetTST for Ctor {
    type Target = ust::Ctor;

    fn forget_tst(&self) -> Self::Target {
        let Ctor { span, doc, name, params, typ } = self;

        ust::Ctor {
            span: *span,
            doc: doc.clone(),
            name: name.clone(),
            params: params.forget_tst(),
            typ: typ.forget_tst(),
        }
    }
}

impl ForgetTST for Dtor {
    type Target = ust::Dtor;

    fn forget_tst(&self) -> Self::Target {
        let Dtor { span, doc, name, params, self_param, ret_typ } = self;

        ust::Dtor {
            span: *span,
            doc: doc.clone(),
            name: name.clone(),
            params: params.forget_tst(),
            self_param: self_param.forget_tst(),
            ret_typ: ret_typ.forget_tst(),
        }
    }
}

impl ForgetTST for Def {
    type Target = ust::Def;

    fn forget_tst(&self) -> Self::Target {
        let Def { span, doc, name, attr, params, self_param, ret_typ, body } = self;

        ust::Def {
            span: *span,
            doc: doc.clone(),
            name: name.clone(),
            attr: attr.clone(),
            params: params.forget_tst(),
            self_param: self_param.forget_tst(),
            ret_typ: ret_typ.forget_tst(),
            body: body.forget_tst(),
        }
    }
}

impl ForgetTST for Codef {
    type Target = ust::Codef;

    fn forget_tst(&self) -> Self::Target {
        let Codef { span, doc, name, attr, params, typ, body } = self;

        ust::Codef {
            span: *span,
            doc: doc.clone(),
            name: name.clone(),
            attr: attr.clone(),
            params: params.forget_tst(),
            typ: typ.forget_tst(),
            body: body.forget_tst(),
        }
    }
}

impl ForgetTST for Match {
    type Target = ust::Match;

    fn forget_tst(&self) -> Self::Target {
        let Match { span, cases, omit_absurd } = self;

        ust::Match { span: *span, cases: cases.forget_tst(), omit_absurd: *omit_absurd }
    }
}

impl ForgetTST for Case {
    type Target = ust::Case;

    fn forget_tst(&self) -> Self::Target {
        let Case { span, name, params, body } = self;

        ust::Case {
            span: *span,
            name: name.clone(),
            params: params.forget_tst(),
            body: body.forget_tst(),
        }
    }
}

impl ForgetTST for SelfParam {
    type Target = ust::SelfParam;

    fn forget_tst(&self) -> Self::Target {
        let SelfParam { info, name, typ } = self;

        ust::SelfParam { info: *info, name: name.clone(), typ: typ.forget_tst() }
    }
}

impl ForgetTST for TypApp {
    type Target = ust::TypApp;

    fn forget_tst(&self) -> Self::Target {
        let TypApp { span, info, name, args } = self;

        ust::TypApp {
            span: *span,
            info: info.forget_tst(),
            name: name.clone(),
            args: args.forget_tst(),
        }
    }
}

impl ForgetTST for Exp {
    type Target = ust::Exp;

    fn forget_tst(&self) -> Self::Target {
        match self {
            Exp::Variable(e) => ust::Exp::Variable(e.forget_tst()),
            Exp::TypCtor(e) => ust::Exp::TypCtor(e.forget_tst()),
            Exp::Call(e) => ust::Exp::Call(e.forget_tst()),
            Exp::DotCall(e) => ust::Exp::DotCall(e.forget_tst()),
            Exp::Anno(e) => ust::Exp::Anno(e.forget_tst()),
            Exp::Type(e) => ust::Exp::Type(e.forget_tst()),
            Exp::LocalMatch(e) => ust::Exp::LocalMatch(e.forget_tst()),
            Exp::LocalComatch(e) => ust::Exp::LocalComatch(e.forget_tst()),
            Exp::Hole(e) => ust::Exp::Hole(e.forget_tst()),
        }
    }
}

impl ForgetTST for Variable {
    type Target = ust::Variable;

    fn forget_tst(&self) -> Self::Target {
        let Variable { span, info, name, ctx: _, idx } = self;
        ust::Variable {
            span: *span,
            info: info.forget_tst(),
            name: name.clone(),
            ctx: (),
            idx: *idx,
        }
    }
}

impl ForgetTST for TypCtor {
    type Target = ust::TypCtor;

    fn forget_tst(&self) -> Self::Target {
        let TypCtor { span, info, name, args } = self;
        ust::TypCtor {
            span: *span,
            info: info.forget_tst(),
            name: name.clone(),
            args: args.forget_tst(),
        }
    }
}

impl ForgetTST for Call {
    type Target = ust::Call;

    fn forget_tst(&self) -> Self::Target {
        let Call { span, info, name, args } = self;
        ust::Call {
            span: *span,
            info: info.forget_tst(),
            name: name.clone(),
            args: args.forget_tst(),
        }
    }
}

impl ForgetTST for DotCall {
    type Target = ust::DotCall;

    fn forget_tst(&self) -> Self::Target {
        let DotCall { span, info, exp, name, args } = self;
        ust::DotCall {
            span: *span,
            info: info.forget_tst(),
            exp: exp.forget_tst(),
            name: name.clone(),
            args: args.forget_tst(),
        }
    }
}

impl ForgetTST for Anno {
    type Target = ust::Anno;

    fn forget_tst(&self) -> Self::Target {
        let Anno { span, info, exp, typ } = self;
        ust::Anno {
            span: *span,
            info: info.forget_tst(),
            exp: exp.forget_tst(),
            typ: typ.forget_tst(),
        }
    }
}

impl ForgetTST for Type {
    type Target = ust::Type;

    fn forget_tst(&self) -> Self::Target {
        let Type { span, info } = self;
        ust::Type { span: *span, info: info.forget_tst() }
    }
}

impl ForgetTST for LocalMatch {
    type Target = ust::LocalMatch;

    fn forget_tst(&self) -> Self::Target {
        let LocalMatch { span, info, ctx: _, name, on_exp, motive, ret_typ, body } = self;
        ust::LocalMatch {
            span: *span,
            info: info.forget_tst(),
            ctx: (),
            name: name.clone(),
            on_exp: on_exp.forget_tst(),
            motive: motive.forget_tst(),
            ret_typ: ret_typ.forget_tst(),
            body: body.forget_tst(),
        }
    }
}

impl ForgetTST for LocalComatch {
    type Target = ust::LocalComatch;

    fn forget_tst(&self) -> Self::Target {
        let LocalComatch { span, info, ctx: _, name, is_lambda_sugar, body } = self;

        ust::LocalComatch {
            span: *span,
            info: info.forget_tst(),
            ctx: (),
            name: name.clone(),
            is_lambda_sugar: *is_lambda_sugar,
            body: body.forget_tst(),
        }
    }
}

impl ForgetTST for Hole {
    type Target = ust::Hole;

    fn forget_tst(&self) -> Self::Target {
        let Hole { span, info } = self;
        ust::Hole { span: *span, info: info.forget_tst() }
    }
}

impl ForgetTST for Motive {
    type Target = ust::Motive;

    fn forget_tst(&self) -> Self::Target {
        let Motive { span, param, ret_typ } = self;

        ust::Motive { span: *span, param: param.forget_tst(), ret_typ: ret_typ.forget_tst() }
    }
}

impl ForgetTST for Telescope {
    type Target = ust::Telescope;

    fn forget_tst(&self) -> Self::Target {
        let Telescope { params } = self;

        ust::Telescope { params: params.forget_tst() }
    }
}

impl ForgetTST for Param {
    type Target = ust::Param;

    fn forget_tst(&self) -> Self::Target {
        let Param { name, typ } = self;

        ust::Param { name: name.clone(), typ: typ.forget_tst() }
    }
}

impl ForgetTST for TelescopeInst {
    type Target = ust::TelescopeInst;

    fn forget_tst(&self) -> Self::Target {
        let TelescopeInst { params } = self;

        ust::TelescopeInst { params: params.forget_tst() }
    }
}

impl ForgetTST for ParamInst {
    type Target = ust::ParamInst;

    fn forget_tst(&self) -> Self::Target {
        let ParamInst { span, info, name, typ } = self;

        ust::ParamInst {
            span: *span,
            info: info.forget_tst(),
            name: name.clone(),
            typ: typ.forget_tst(),
        }
    }
}

impl ForgetTST for Args {
    type Target = ust::Args;

    fn forget_tst(&self) -> Self::Target {
        ust::Args { args: self.args.forget_tst() }
    }
}

impl ForgetTST for Typ {
    type Target = ();

    fn forget_tst(&self) -> Self::Target {}
}

impl ForgetTST for TypeInfo {
    type Target = ();

    fn forget_tst(&self) -> Self::Target {}
}

impl ForgetTST for TypeAppInfo {
    type Target = ();

    fn forget_tst(&self) -> Self::Target {}
}
