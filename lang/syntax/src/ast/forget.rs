//! Convert a typed syntax tree to an untyped tree

use std::rc::Rc;

use crate::tst;
use crate::ust;

pub trait Forget {
    type Target;

    fn forget(&self) -> Self::Target;
}

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
        let tst::Decls { map, source } = self;

        ust::Decls {
            map: map.iter().map(|(name, decl)| (name.clone(), decl.forget())).collect(),
            source: source.clone(),
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
        let tst::Data { info, name, typ, ctors, impl_block } = self;

        ust::Data {
            info: info.forget(),
            name: name.clone(),
            typ: typ.forget(),
            ctors: ctors.clone(),
            impl_block: impl_block.forget(),
        }
    }
}

impl Forget for tst::Codata {
    type Target = ust::Codata;

    fn forget(&self) -> Self::Target {
        let tst::Codata { info, name, typ, dtors, impl_block } = self;

        ust::Codata {
            info: info.forget(),
            name: name.clone(),
            typ: typ.forget(),
            dtors: dtors.clone(),
            impl_block: impl_block.forget(),
        }
    }
}

impl Forget for tst::Impl {
    type Target = ust::Impl;

    fn forget(&self) -> Self::Target {
        let tst::Impl { info, name, defs } = self;

        ust::Impl { info: info.forget(), name: name.clone(), defs: defs.clone() }
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
        let tst::Ctor { info, name, params, typ } = self;

        ust::Ctor {
            info: info.forget(),
            name: name.clone(),
            params: params.forget(),
            typ: typ.forget(),
        }
    }
}

impl Forget for tst::Dtor {
    type Target = ust::Dtor;

    fn forget(&self) -> Self::Target {
        let tst::Dtor { info, name, params, on_typ, in_typ } = self;

        ust::Dtor {
            info: info.forget(),
            name: name.clone(),
            params: params.forget(),
            on_typ: on_typ.forget(),
            in_typ: in_typ.forget(),
        }
    }
}

impl Forget for tst::Def {
    type Target = ust::Def;

    fn forget(&self) -> Self::Target {
        let tst::Def { info, name, params, on_typ, in_typ, body } = self;

        ust::Def {
            info: info.forget(),
            name: name.clone(),
            params: params.forget(),
            on_typ: on_typ.forget(),
            in_typ: in_typ.forget(),
            body: body.forget(),
        }
    }
}

impl Forget for tst::Codef {
    type Target = ust::Codef;

    fn forget(&self) -> Self::Target {
        let tst::Codef { info, name, params, typ, body } = self;

        ust::Codef {
            info: info.forget(),
            name: name.clone(),
            params: params.forget(),
            typ: typ.forget(),
            body: body.forget(),
        }
    }
}

impl Forget for tst::Match {
    type Target = ust::Match;

    fn forget(&self) -> Self::Target {
        let tst::Match { info, cases } = self;

        ust::Match { info: info.forget(), cases: cases.forget() }
    }
}

impl Forget for tst::Comatch {
    type Target = ust::Comatch;

    fn forget(&self) -> Self::Target {
        let tst::Comatch { info, cases } = self;

        ust::Comatch { info: info.forget(), cases: cases.forget() }
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
        let tst::Cocase { info, name, args, body } = self;

        ust::Cocase {
            info: info.forget(),
            name: name.clone(),
            args: args.forget(),
            body: body.forget(),
        }
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
            tst::Exp::Var { info, name, idx } => {
                ust::Exp::Var { info: info.forget(), name: name.clone(), idx: *idx }
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
            tst::Exp::Match { info, name, on_exp, in_typ: _, body } => ust::Exp::Match {
                info: info.forget(),
                name: name.clone(),
                on_exp: on_exp.forget(),
                in_typ: (),
                body: body.forget(),
            },
            tst::Exp::Comatch { info, name, body } => {
                ust::Exp::Comatch { info: info.forget(), name: name.clone(), body: body.forget() }
            }
            tst::Exp::Hole {} => todo!(),
        }
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
        let tst::ParamInst { info, name, typ: _ } = self;

        ust::ParamInst { info: info.forget(), name: name.clone(), typ: () }
    }
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
        let tst::TypeInfo { span, .. } = self;

        ust::Info { span: *span }
    }
}

impl Forget for tst::TypeAppInfo {
    type Target = ust::Info;

    fn forget(&self) -> Self::Target {
        let tst::TypeAppInfo { span, .. } = self;

        ust::Info { span: *span }
    }
}

impl<T: Forget> Forget for Rc<T> {
    type Target = Rc<T::Target>;

    fn forget(&self) -> Self::Target {
        Rc::new(T::forget(self))
    }
}

impl<T: Forget> Forget for Option<T> {
    type Target = Option<T::Target>;

    fn forget(&self) -> Self::Target {
        self.as_ref().map(Forget::forget)
    }
}

impl<T: Forget> Forget for Vec<T> {
    type Target = Vec<T::Target>;

    fn forget(&self) -> Self::Target {
        self.iter().map(Forget::forget).collect()
    }
}
