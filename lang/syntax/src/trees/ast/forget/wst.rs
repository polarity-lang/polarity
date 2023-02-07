//! Convert a typed syntax tree to an untyped tree

use crate::common::*;
use crate::ust;
use crate::wst;

impl Forget for wst::Prg {
    type Target = ust::Prg;

    fn forget(&self) -> Self::Target {
        let wst::Prg { decls, exp } = self;

        ust::Prg { decls: decls.forget(), exp: exp.forget() }
    }
}

impl Forget for wst::Decls {
    type Target = ust::Decls;

    fn forget(&self) -> Self::Target {
        let wst::Decls { map, source } = self;

        ust::Decls {
            map: map.iter().map(|(name, decl)| (name.clone(), decl.forget())).collect(),
            source: source.clone(),
        }
    }
}

impl Forget for wst::Decl {
    type Target = ust::Decl;

    fn forget(&self) -> Self::Target {
        match self {
            wst::Decl::Data(data) => ust::Decl::Data(data.forget()),
            wst::Decl::Codata(codata) => ust::Decl::Codata(codata.forget()),
            wst::Decl::Ctor(ctor) => ust::Decl::Ctor(ctor.forget()),
            wst::Decl::Dtor(dtor) => ust::Decl::Dtor(dtor.forget()),
            wst::Decl::Def(def) => ust::Decl::Def(def.forget()),
            wst::Decl::Codef(codef) => ust::Decl::Codef(codef.forget()),
        }
    }
}

impl Forget for wst::Data {
    type Target = ust::Data;

    fn forget(&self) -> Self::Target {
        let wst::Data { info, name, ignored, typ, ctors } = self;

        ust::Data {
            info: info.forget(),
            name: name.clone(),
            ignored: *ignored,
            typ: typ.forget(),
            ctors: ctors.clone(),
        }
    }
}

impl Forget for wst::Codata {
    type Target = ust::Codata;

    fn forget(&self) -> Self::Target {
        let wst::Codata { info, name, ignored, typ, dtors } = self;

        ust::Codata {
            info: info.forget(),
            name: name.clone(),
            ignored: *ignored,
            typ: typ.forget(),
            dtors: dtors.clone(),
        }
    }
}

impl Forget for wst::TypAbs {
    type Target = ust::TypAbs;

    fn forget(&self) -> Self::Target {
        let wst::TypAbs { params } = self;

        ust::TypAbs { params: params.forget() }
    }
}

impl Forget for wst::Ctor {
    type Target = ust::Ctor;

    fn forget(&self) -> Self::Target {
        let wst::Ctor { info, name, params, typ } = self;

        ust::Ctor {
            info: info.forget(),
            name: name.clone(),
            params: params.forget(),
            typ: typ.forget(),
        }
    }
}

impl Forget for wst::Dtor {
    type Target = ust::Dtor;

    fn forget(&self) -> Self::Target {
        let wst::Dtor { info, name, params, self_param, ret_typ } = self;

        ust::Dtor {
            info: info.forget(),
            name: name.clone(),
            params: params.forget(),
            self_param: self_param.forget(),
            ret_typ: ret_typ.forget(),
        }
    }
}

impl Forget for wst::Def {
    type Target = ust::Def;

    fn forget(&self) -> Self::Target {
        let wst::Def { info, name, ignored, params, self_param, ret_typ, body } = self;

        ust::Def {
            info: info.forget(),
            name: name.clone(),
            ignored: *ignored,
            params: params.forget(),
            self_param: self_param.forget(),
            ret_typ: ret_typ.forget(),
            body: body.forget(),
        }
    }
}

impl Forget for wst::Codef {
    type Target = ust::Codef;

    fn forget(&self) -> Self::Target {
        let wst::Codef { info, name, ignored, params, typ, body } = self;

        ust::Codef {
            info: info.forget(),
            name: name.clone(),
            ignored: *ignored,
            params: params.forget(),
            typ: typ.forget(),
            body: body.forget(),
        }
    }
}

impl Forget for wst::Match {
    type Target = ust::Match;

    fn forget(&self) -> Self::Target {
        let wst::Match { info, cases } = self;

        ust::Match { info: info.forget(), cases: cases.forget() }
    }
}

impl Forget for wst::Comatch {
    type Target = ust::Comatch;

    fn forget(&self) -> Self::Target {
        let wst::Comatch { info, cases } = self;

        ust::Comatch { info: info.forget(), cases: cases.forget() }
    }
}

impl Forget for wst::Case {
    type Target = ust::Case;

    fn forget(&self) -> Self::Target {
        let wst::Case { info, name, args, body } = self;

        ust::Case {
            info: info.forget(),
            name: name.clone(),
            args: args.forget(),
            body: body.forget(),
        }
    }
}

impl Forget for wst::Cocase {
    type Target = ust::Cocase;

    fn forget(&self) -> Self::Target {
        let wst::Cocase { info, name, params: args, body } = self;

        ust::Cocase {
            info: info.forget(),
            name: name.clone(),
            params: args.forget(),
            body: body.forget(),
        }
    }
}

impl Forget for wst::SelfParam {
    type Target = ust::SelfParam;

    fn forget(&self) -> Self::Target {
        let wst::SelfParam { info, name, typ } = self;

        ust::SelfParam { info: info.forget(), name: name.clone(), typ: typ.forget() }
    }
}

impl Forget for wst::TypApp {
    type Target = ust::TypApp;

    fn forget(&self) -> Self::Target {
        let wst::TypApp { info, name, args } = self;

        ust::TypApp { info: info.forget(), name: name.clone(), args: args.forget() }
    }
}

impl Forget for wst::Exp {
    type Target = ust::Exp;

    fn forget(&self) -> Self::Target {
        match self {
            wst::Exp::Var { info, name, idx } => {
                ust::Exp::Var { info: info.forget(), name: name.clone(), idx: *idx }
            }
            wst::Exp::TypCtor { info, name, args } => {
                ust::Exp::TypCtor { info: info.forget(), name: name.clone(), args: args.forget() }
            }
            wst::Exp::Ctor { info, name, args } => {
                ust::Exp::Ctor { info: info.forget(), name: name.clone(), args: args.forget() }
            }
            wst::Exp::Dtor { info, exp, name, args } => ust::Exp::Dtor {
                info: info.forget(),
                exp: exp.forget(),
                name: name.clone(),
                args: args.forget(),
            },
            wst::Exp::Anno { info, exp, typ } => {
                ust::Exp::Anno { info: info.forget(), exp: exp.forget(), typ: typ.forget() }
            }
            wst::Exp::Type { info } => ust::Exp::Type { info: info.forget() },
            wst::Exp::Match { info, name, on_exp, motive, ret_typ: _, body } => ust::Exp::Match {
                info: info.forget(),
                name: name.clone(),
                on_exp: on_exp.forget(),
                motive: motive.forget(),
                ret_typ: (),
                body: body.forget(),
            },
            wst::Exp::Comatch { info, name, body } => {
                ust::Exp::Comatch { info: info.forget(), name: name.clone(), body: body.forget() }
            }
            wst::Exp::Hole { info } => ust::Exp::Hole { info: info.forget() },
        }
    }
}

impl Forget for wst::Motive {
    type Target = ust::Motive;

    fn forget(&self) -> Self::Target {
        let wst::Motive { info, param, ret_typ } = self;

        ust::Motive { info: info.forget(), param: param.forget(), ret_typ: ret_typ.forget() }
    }
}

impl Forget for wst::Telescope {
    type Target = ust::Telescope;

    fn forget(&self) -> Self::Target {
        let wst::Telescope { params } = self;

        ust::Telescope { params: params.forget() }
    }
}

impl Forget for wst::Param {
    type Target = ust::Param;

    fn forget(&self) -> Self::Target {
        let wst::Param { name, typ } = self;

        ust::Param { name: name.clone(), typ: typ.forget() }
    }
}

impl Forget for wst::TelescopeInst {
    type Target = ust::TelescopeInst;

    fn forget(&self) -> Self::Target {
        let wst::TelescopeInst { params } = self;

        ust::TelescopeInst { params: params.forget() }
    }
}

impl Forget for wst::ParamInst {
    type Target = ust::ParamInst;

    fn forget(&self) -> Self::Target {
        let wst::ParamInst { info, name, typ: _ } = self;

        ust::ParamInst { info: info.forget(), name: name.clone(), typ: () }
    }
}

impl Forget for wst::Info {
    type Target = ust::Info;

    fn forget(&self) -> Self::Target {
        let wst::Info { span } = self;

        ust::Info { span: *span }
    }
}

impl Forget for wst::TypeInfo {
    type Target = ust::Info;

    fn forget(&self) -> Self::Target {
        let wst::TypeInfo { span, .. } = self;

        ust::Info { span: *span }
    }
}

impl Forget for wst::TypeAppInfo {
    type Target = ust::Info;

    fn forget(&self) -> Self::Target {
        let wst::TypeAppInfo { span, .. } = self;

        ust::Info { span: *span }
    }
}
