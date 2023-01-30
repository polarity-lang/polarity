//! Convert a typed syntax tree to a representation suitable for program transformations

use crate::common::*;
use crate::tst;
use crate::wst;

impl Forget for tst::Prg {
    type Target = wst::Prg;

    fn forget(&self) -> Self::Target {
        let tst::Prg { decls, exp } = self;

        wst::Prg { decls: decls.forget(), exp: exp.forget() }
    }
}

impl Forget for tst::Decls {
    type Target = wst::Decls;

    fn forget(&self) -> Self::Target {
        let tst::Decls { map, source } = self;

        wst::Decls {
            map: map.iter().map(|(name, decl)| (name.clone(), decl.forget())).collect(),
            source: source.clone(),
        }
    }
}

impl Forget for tst::Decl {
    type Target = wst::Decl;

    fn forget(&self) -> Self::Target {
        match self {
            tst::Decl::Data(data) => wst::Decl::Data(data.forget()),
            tst::Decl::Codata(codata) => wst::Decl::Codata(codata.forget()),
            tst::Decl::Ctor(ctor) => wst::Decl::Ctor(ctor.forget()),
            tst::Decl::Dtor(dtor) => wst::Decl::Dtor(dtor.forget()),
            tst::Decl::Def(def) => wst::Decl::Def(def.forget()),
            tst::Decl::Codef(codef) => wst::Decl::Codef(codef.forget()),
        }
    }
}

impl Forget for tst::Data {
    type Target = wst::Data;

    fn forget(&self) -> Self::Target {
        let tst::Data { info, name, typ, ctors, impl_block } = self;

        wst::Data {
            info: info.forget(),
            name: name.clone(),
            typ: typ.forget(),
            ctors: ctors.clone(),
            impl_block: impl_block.forget(),
        }
    }
}

impl Forget for tst::Codata {
    type Target = wst::Codata;

    fn forget(&self) -> Self::Target {
        let tst::Codata { info, name, typ, dtors, impl_block } = self;

        wst::Codata {
            info: info.forget(),
            name: name.clone(),
            typ: typ.forget(),
            dtors: dtors.clone(),
            impl_block: impl_block.forget(),
        }
    }
}

impl Forget for tst::Impl {
    type Target = wst::Impl;

    fn forget(&self) -> Self::Target {
        let tst::Impl { info, name, defs } = self;

        wst::Impl { info: info.forget(), name: name.clone(), defs: defs.clone() }
    }
}

impl Forget for tst::TypAbs {
    type Target = wst::TypAbs;

    fn forget(&self) -> Self::Target {
        let tst::TypAbs { params } = self;

        wst::TypAbs { params: params.forget() }
    }
}

impl Forget for tst::Ctor {
    type Target = wst::Ctor;

    fn forget(&self) -> Self::Target {
        let tst::Ctor { info, name, params, typ } = self;

        wst::Ctor {
            info: info.forget(),
            name: name.clone(),
            params: params.forget(),
            typ: typ.forget(),
        }
    }
}

impl Forget for tst::Dtor {
    type Target = wst::Dtor;

    fn forget(&self) -> Self::Target {
        let tst::Dtor { info, name, params, self_param, ret_typ } = self;

        wst::Dtor {
            info: info.forget(),
            name: name.clone(),
            params: params.forget(),
            self_param: self_param.forget(),
            ret_typ: ret_typ.forget(),
        }
    }
}

impl Forget for tst::Def {
    type Target = wst::Def;

    fn forget(&self) -> Self::Target {
        let tst::Def { info, name, params, self_param, ret_typ, body } = self;

        wst::Def {
            info: info.forget(),
            name: name.clone(),
            params: params.forget(),
            self_param: self_param.forget(),
            ret_typ: ret_typ.forget(),
            body: body.forget(),
        }
    }
}

impl Forget for tst::Codef {
    type Target = wst::Codef;

    fn forget(&self) -> Self::Target {
        let tst::Codef { info, name, params, typ, body } = self;

        wst::Codef {
            info: info.forget(),
            name: name.clone(),
            params: params.forget(),
            typ: typ.forget(),
            body: body.forget(),
        }
    }
}

impl Forget for tst::Match {
    type Target = wst::Match;

    fn forget(&self) -> Self::Target {
        let tst::Match { info, cases } = self;

        wst::Match { info: info.forget(), cases: cases.forget() }
    }
}

impl Forget for tst::Comatch {
    type Target = wst::Comatch;

    fn forget(&self) -> Self::Target {
        let tst::Comatch { info, cases } = self;

        wst::Comatch { info: info.forget(), cases: cases.forget() }
    }
}

impl Forget for tst::Case {
    type Target = wst::Case;

    fn forget(&self) -> Self::Target {
        let tst::Case { info, name, args, body } = self;

        wst::Case {
            info: info.forget(),
            name: name.clone(),
            args: args.forget(),
            body: body.forget(),
        }
    }
}

impl Forget for tst::Cocase {
    type Target = wst::Cocase;

    fn forget(&self) -> Self::Target {
        let tst::Cocase { info, name, params: args, body } = self;

        wst::Cocase {
            info: info.forget(),
            name: name.clone(),
            params: args.forget(),
            body: body.forget(),
        }
    }
}

impl Forget for tst::SelfParam {
    type Target = wst::SelfParam;

    fn forget(&self) -> Self::Target {
        let tst::SelfParam { info, name, typ } = self;

        wst::SelfParam { info: info.forget(), name: name.clone(), typ: typ.forget() }
    }
}

impl Forget for tst::TypApp {
    type Target = wst::TypApp;

    fn forget(&self) -> Self::Target {
        let tst::TypApp { info, name, args } = self;

        wst::TypApp { info: info.forget(), name: name.clone(), args: args.forget() }
    }
}

impl Forget for tst::Exp {
    type Target = wst::Exp;

    fn forget(&self) -> Self::Target {
        match self {
            tst::Exp::Var { info, name, idx } => {
                wst::Exp::Var { info: info.forget(), name: name.clone(), idx: *idx }
            }
            tst::Exp::TypCtor { info, name, args } => {
                wst::Exp::TypCtor { info: info.forget(), name: name.clone(), args: args.forget() }
            }
            tst::Exp::Ctor { info, name, args } => {
                wst::Exp::Ctor { info: info.forget(), name: name.clone(), args: args.forget() }
            }
            tst::Exp::Dtor { info, exp, name, args } => wst::Exp::Dtor {
                info: info.forget(),
                exp: exp.forget(),
                name: name.clone(),
                args: args.forget(),
            },
            tst::Exp::Anno { info, exp, typ } => {
                wst::Exp::Anno { info: info.forget(), exp: exp.forget(), typ: typ.forget() }
            }
            tst::Exp::Type { info } => wst::Exp::Type { info: info.forget() },
            tst::Exp::Match { info, name, on_exp, in_typ, body } => wst::Exp::Match {
                info: info.forget(),
                name: name.clone(),
                on_exp: on_exp.forget(),
                in_typ: in_typ.forget(),
                body: body.forget(),
            },
            tst::Exp::Comatch { info, name, body } => {
                wst::Exp::Comatch { info: info.forget(), name: name.clone(), body: body.forget() }
            }
        }
    }
}

impl Forget for tst::Telescope {
    type Target = wst::Telescope;

    fn forget(&self) -> Self::Target {
        let tst::Telescope { params } = self;

        wst::Telescope { params: params.forget() }
    }
}

impl Forget for tst::Param {
    type Target = wst::Param;

    fn forget(&self) -> Self::Target {
        let tst::Param { name, typ } = self;

        wst::Param { name: name.clone(), typ: typ.forget() }
    }
}

impl Forget for tst::TelescopeInst {
    type Target = wst::TelescopeInst;

    fn forget(&self) -> Self::Target {
        let tst::TelescopeInst { params } = self;

        wst::TelescopeInst { params: params.forget() }
    }
}

impl Forget for tst::ParamInst {
    type Target = wst::ParamInst;

    fn forget(&self) -> Self::Target {
        let tst::ParamInst { info, name, typ } = self;

        wst::ParamInst { info: info.forget(), name: name.clone(), typ: typ.forget() }
    }
}

impl Forget for tst::Typ {
    type Target = wst::Typ;

    fn forget(&self) -> Self::Target {
        wst::Typ::from(self.as_exp().forget())
    }
}

impl Forget for tst::Info {
    type Target = wst::Info;

    fn forget(&self) -> Self::Target {
        let tst::Info { span } = self;

        wst::Info { span: *span }
    }
}

impl Forget for tst::TypeInfo {
    type Target = wst::TypeInfo;

    fn forget(&self) -> Self::Target {
        let tst::TypeInfo { typ, span } = self;

        wst::TypeInfo { typ: typ.forget(), span: *span }
    }
}

impl Forget for tst::TypeAppInfo {
    type Target = wst::TypeAppInfo;

    fn forget(&self) -> Self::Target {
        let tst::TypeAppInfo { typ, span, .. } = self;

        wst::TypeAppInfo { typ: typ.forget(), span: *span }
    }
}
