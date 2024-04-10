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
        let Data { info, doc, name, attr, typ, ctors } = self;

        ust::Data {
            info: *info,
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
        let Let { info, doc, name, attr, params, typ, body } = self;

        ust::Let {
            info: *info,
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
        let Codata { info, doc, name, attr, typ, dtors } = self;

        ust::Codata {
            info: *info,
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
        let Ctor { info, doc, name, params, typ } = self;

        ust::Ctor {
            info: *info,
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
        let Dtor { info, doc, name, params, self_param, ret_typ } = self;

        ust::Dtor {
            info: *info,
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
        let Def { info, doc, name, attr, params, self_param, ret_typ, body } = self;

        ust::Def {
            info: *info,
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
        let Codef { info, doc, name, attr, params, typ, body } = self;

        ust::Codef {
            info: *info,
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
        let Match { info, cases, omit_absurd } = self;

        ust::Match { info: *info, cases: cases.forget_tst(), omit_absurd: *omit_absurd }
    }
}

impl ForgetTST for Case {
    type Target = ust::Case;

    fn forget_tst(&self) -> Self::Target {
        let Case { info, name, args, body } = self;

        ust::Case {
            info: *info,
            name: name.clone(),
            args: args.forget_tst(),
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
        let TypApp { info: _, span, name, args } = self;

        ust::TypApp { info: (), span: span.clone(), name: name.clone(), args: args.forget_tst() }
    }
}

impl ForgetTST for Exp {
    type Target = ust::Exp;

    fn forget_tst(&self) -> Self::Target {
        match self {
            Exp::Var { info: _, span, name, ctx: _, idx } => ust::Exp::Var {
                info: (),
                span: span.clone(),
                name: name.clone(),
                ctx: (),
                idx: *idx,
            },
            Exp::TypCtor { info: _, span, name, args } => ust::Exp::TypCtor {
                info: (),
                span: span.clone(),
                name: name.clone(),
                args: args.forget_tst(),
            },
            Exp::Ctor { info: _, span, name, args } => ust::Exp::Ctor {
                info: (),
                span: span.clone(),
                name: name.clone(),
                args: args.forget_tst(),
            },
            Exp::Dtor { info: _, span, exp, name, args } => ust::Exp::Dtor {
                info: (),
                span: span.clone(),
                exp: exp.forget_tst(),
                name: name.clone(),
                args: args.forget_tst(),
            },
            Exp::Anno { info: _, span, exp, typ } => ust::Exp::Anno {
                info: (),
                span: span.clone(),
                exp: exp.forget_tst(),
                typ: typ.forget_tst(),
            },
            Exp::Type { info: _, span } => ust::Exp::Type { info: (), span: span.clone() },
            Exp::Match { info: _, span, ctx: _, name, on_exp, motive, ret_typ, body } => {
                ust::Exp::Match {
                    info: (),
                    span: span.clone(),
                    ctx: (),
                    name: name.clone(),
                    on_exp: on_exp.forget_tst(),
                    motive: motive.forget_tst(),
                    ret_typ: ret_typ.forget_tst(),
                    body: body.forget_tst(),
                }
            }
            Exp::Comatch { info: _, span, ctx: _, name, is_lambda_sugar, body } => {
                ust::Exp::Comatch {
                    info: (),
                    span: span.clone(),
                    ctx: (),
                    name: name.clone(),
                    is_lambda_sugar: *is_lambda_sugar,
                    body: body.forget_tst(),
                }
            }
            Exp::Hole { info: _, span } => ust::Exp::Hole { info: (), span: span.clone() },
        }
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
        let ParamInst { info: _, span, name, typ } = self;

        ust::ParamInst { info: (), span: span.clone(), name: name.clone(), typ: typ.forget_tst() }
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
