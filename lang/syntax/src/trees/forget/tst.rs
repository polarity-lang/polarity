//! Convert a typed syntax tree to a representation suitable for program transformations

use crate::common::*;
use crate::tst;
use crate::ust;

impl Forget for tst::Prg {
    type Target = ust::Prg;

    fn forget(&self) -> Self::Target {
        let tst::Prg { decls, exp } = self;

        ust::Prg { decls: decls.forget(), exp: exp.forget() }
    }
}

impl Forget for tst::Decls {
    type Target = ust::Decls;

    fn forget(&self) -> Self::Target {
        let tst::Decls { map, lookup_table } = self;

        ust::Decls {
            map: map.iter().map(|(name, decl)| (name.clone(), decl.forget())).collect(),
            lookup_table: lookup_table.clone(),
        }
    }
}

impl Forget for tst::Decl {
    type Target = ust::Decl;

    fn forget(&self) -> Self::Target {
        match self {
            tst::Decl::Data(data) => ust::Decl::Data(data.forget()),
            tst::Decl::Codata(codata) => ust::Decl::Codata(codata.forget()),
            tst::Decl::Ctor(ctor) => ust::Decl::Ctor(ctor.forget()),
            tst::Decl::Dtor(dtor) => ust::Decl::Dtor(dtor.forget()),
            tst::Decl::Def(def) => ust::Decl::Def(def.forget()),
            tst::Decl::Codef(codef) => ust::Decl::Codef(codef.forget()),
        }
    }
}

impl Forget for tst::Data {
    type Target = ust::Data;

    fn forget(&self) -> Self::Target {
        let tst::Data { info, doc, name, hidden, typ, ctors } = self;

        ust::Data {
            info: info.forget(),
            name: name.clone(),
            doc: doc.clone(),
            hidden: *hidden,
            typ: typ.forget(),
            ctors: ctors.clone(),
        }
    }
}

impl Forget for tst::Codata {
    type Target = ust::Codata;

    fn forget(&self) -> Self::Target {
        let tst::Codata { info, doc, name, hidden, typ, dtors } = self;

        ust::Codata {
            info: info.forget(),
            doc: doc.clone(),
            name: name.clone(),
            hidden: *hidden,
            typ: typ.forget(),
            dtors: dtors.clone(),
        }
    }
}

impl Forget for tst::TypAbs {
    type Target = ust::TypAbs;

    fn forget(&self) -> Self::Target {
        let tst::TypAbs { params } = self;

        ust::TypAbs { params: params.forget() }
    }
}

impl Forget for tst::Ctor {
    type Target = ust::Ctor;

    fn forget(&self) -> Self::Target {
        let tst::Ctor { info, doc, name, params, typ } = self;

        ust::Ctor {
            info: info.forget(),
            doc: doc.clone(),
            name: name.clone(),
            params: params.forget(),
            typ: typ.forget(),
        }
    }
}

impl Forget for tst::Dtor {
    type Target = ust::Dtor;

    fn forget(&self) -> Self::Target {
        let tst::Dtor { info, doc, name, params, self_param, ret_typ } = self;

        ust::Dtor {
            info: info.forget(),
            doc: doc.clone(),
            name: name.clone(),
            params: params.forget(),
            self_param: self_param.forget(),
            ret_typ: ret_typ.forget(),
        }
    }
}

impl Forget for tst::Def {
    type Target = ust::Def;

    fn forget(&self) -> Self::Target {
        let tst::Def { info, doc, name, hidden, params, self_param, ret_typ, body } = self;

        ust::Def {
            info: info.forget(),
            doc: doc.clone(),
            name: name.clone(),
            hidden: *hidden,
            params: params.forget(),
            self_param: self_param.forget(),
            ret_typ: ret_typ.forget(),
            body: body.forget(),
        }
    }
}

impl Forget for tst::Codef {
    type Target = ust::Codef;

    fn forget(&self) -> Self::Target {
        let tst::Codef { info, doc, name, hidden, params, typ, body } = self;

        ust::Codef {
            info: info.forget(),
            doc: doc.clone(),
            name: name.clone(),
            hidden: *hidden,
            params: params.forget(),
            typ: typ.forget(),
            body: body.forget(),
        }
    }
}

impl Forget for tst::Match {
    type Target = ust::Match;

    fn forget(&self) -> Self::Target {
        let tst::Match { info, cases, omit_absurd } = self;

        ust::Match { info: info.forget(), cases: cases.forget(), omit_absurd: *omit_absurd }
    }
}

impl Forget for tst::Comatch {
    type Target = ust::Comatch;

    fn forget(&self) -> Self::Target {
        let tst::Comatch { info, cases, omit_absurd } = self;

        ust::Comatch { info: info.forget(), cases: cases.forget(), omit_absurd: *omit_absurd }
    }
}

impl Forget for tst::Case {
    type Target = ust::Case;

    fn forget(&self) -> Self::Target {
        let tst::Case { info, name, args, body } = self;

        ust::Case {
            info: info.forget(),
            name: name.clone(),
            args: args.forget(),
            body: body.forget(),
        }
    }
}

impl Forget for tst::Cocase {
    type Target = ust::Cocase;

    fn forget(&self) -> Self::Target {
        let tst::Cocase { info, name, params: args, body } = self;

        ust::Cocase {
            info: info.forget(),
            name: name.clone(),
            params: args.forget(),
            body: body.forget(),
        }
    }
}

impl Forget for tst::SelfParam {
    type Target = ust::SelfParam;

    fn forget(&self) -> Self::Target {
        let tst::SelfParam { info, name, typ } = self;

        ust::SelfParam { info: info.forget(), name: name.clone(), typ: typ.forget() }
    }
}

impl Forget for tst::TypApp {
    type Target = ust::TypApp;

    fn forget(&self) -> Self::Target {
        let tst::TypApp { info, name, args } = self;

        ust::TypApp { info: info.forget(), name: name.clone(), args: args.forget() }
    }
}

impl Forget for tst::Exp {
    type Target = ust::Exp;

    fn forget(&self) -> Self::Target {
        match self {
            tst::Exp::Var { info, name, ctx: _, idx } => {
                ust::Exp::Var { info: info.forget(), name: name.clone(), ctx: (), idx: *idx }
            }
            tst::Exp::TypCtor { info, name, args } => {
                ust::Exp::TypCtor { info: info.forget(), name: name.clone(), args: args.forget() }
            }
            tst::Exp::Ctor { info, name, args } => {
                ust::Exp::Ctor { info: info.forget(), name: name.clone(), args: args.forget() }
            }
            tst::Exp::Dtor { info, exp, name, args } => ust::Exp::Dtor {
                info: info.forget(),
                exp: exp.forget(),
                name: name.clone(),
                args: args.forget(),
            },
            tst::Exp::Anno { info, exp, typ } => {
                ust::Exp::Anno { info: info.forget(), exp: exp.forget(), typ: typ.forget() }
            }
            tst::Exp::Type { info } => ust::Exp::Type { info: info.forget() },
            tst::Exp::Match { info, ctx: _, name, on_exp, motive, ret_typ, body } => {
                ust::Exp::Match {
                    info: info.forget(),
                    ctx: (),
                    name: name.clone(),
                    on_exp: on_exp.forget(),
                    motive: motive.forget(),
                    ret_typ: ret_typ.forget(),
                    body: body.forget(),
                }
            }
            tst::Exp::Comatch { info, ctx: _, name, is_lambda_sugar, body } => ust::Exp::Comatch {
                info: info.forget(),
                ctx: (),
                name: name.clone(),
                is_lambda_sugar: *is_lambda_sugar,
                body: body.forget(),
            },
            tst::Exp::Hole { info, kind } => ust::Exp::Hole { info: info.forget(), kind: *kind },
        }
    }
}

impl Forget for tst::Motive {
    type Target = ust::Motive;

    fn forget(&self) -> Self::Target {
        let tst::Motive { info, param, ret_typ } = self;

        ust::Motive { info: info.forget(), param: param.forget(), ret_typ: ret_typ.forget() }
    }
}

impl Forget for tst::Telescope {
    type Target = ust::Telescope;

    fn forget(&self) -> Self::Target {
        let tst::Telescope { params } = self;

        ust::Telescope { params: params.forget() }
    }
}

impl Forget for tst::Param {
    type Target = ust::Param;

    fn forget(&self) -> Self::Target {
        let tst::Param { name, typ } = self;

        ust::Param { name: name.clone(), typ: typ.forget() }
    }
}

impl Forget for tst::TelescopeInst {
    type Target = ust::TelescopeInst;

    fn forget(&self) -> Self::Target {
        let tst::TelescopeInst { params } = self;

        ust::TelescopeInst { params: params.forget() }
    }
}

impl Forget for tst::ParamInst {
    type Target = ust::ParamInst;

    fn forget(&self) -> Self::Target {
        let tst::ParamInst { info, name, typ } = self;

        ust::ParamInst { info: info.forget(), name: name.clone(), typ: typ.forget() }
    }
}

impl Forget for tst::Args {
    type Target = ust::Args;

    fn forget(&self) -> Self::Target {
        ust::Args { args: self.args.forget() }
    }
}

impl Forget for tst::Typ {
    type Target = ();

    fn forget(&self) -> Self::Target {}
}

impl Forget for tst::Info {
    type Target = ust::Info;

    fn forget(&self) -> Self::Target {
        let tst::Info { span } = self;

        ust::Info { span: *span }
    }
}

impl Forget for tst::TypeInfo {
    type Target = ust::Info;

    fn forget(&self) -> Self::Target {
        let tst::TypeInfo { typ: _, span, ctx: _ } = self;

        ust::Info { span: *span }
    }
}

impl Forget for tst::TypeAppInfo {
    type Target = ust::Info;

    fn forget(&self) -> Self::Target {
        let tst::TypeAppInfo { typ: _, span, .. } = self;

        ust::Info { span: *span }
    }
}
