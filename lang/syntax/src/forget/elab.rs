use std::rc::Rc;

use crate::ast;
use crate::elab;

use super::Forget;

impl Forget for elab::Prg {
    type Target = ast::Prg;

    fn forget(&self) -> Self::Target {
        let elab::Prg { decls, exp } = self;

        ast::Prg { decls: decls.forget(), exp: exp.forget() }
    }
}

impl Forget for elab::Decls {
    type Target = ast::Decls;

    fn forget(&self) -> Self::Target {
        let elab::Decls { map, order } = self;

        ast::Decls {
            map: map.iter().map(|(name, decl)| (name.clone(), decl.forget())).collect(),
            order: order.clone(),
        }
    }
}

impl Forget for elab::Decl {
    type Target = ast::Decl;

    fn forget(&self) -> Self::Target {
        match self {
            elab::Decl::Data(data) => ast::Decl::Data(data.forget()),
            elab::Decl::Codata(codata) => ast::Decl::Codata(codata.forget()),
            elab::Decl::Ctor(ctor) => ast::Decl::Ctor(ctor.forget()),
            elab::Decl::Dtor(dtor) => ast::Decl::Dtor(dtor.forget()),
            elab::Decl::Def(def) => ast::Decl::Def(def.forget()),
            elab::Decl::Codef(codef) => ast::Decl::Codef(codef.forget()),
        }
    }
}

impl Forget for elab::Data {
    type Target = ast::Data;

    fn forget(&self) -> Self::Target {
        let elab::Data { info, name, typ, ctors, impl_block } = self;

        ast::Data {
            info: info.forget(),
            name: name.clone(),
            typ: Rc::new(typ.forget()),
            ctors: ctors.clone(),
            impl_block: impl_block.forget(),
        }
    }
}

impl Forget for elab::Codata {
    type Target = ast::Codata;

    fn forget(&self) -> Self::Target {
        let elab::Codata { info, name, typ, dtors, impl_block } = self;

        ast::Codata {
            info: info.forget(),
            name: name.clone(),
            typ: Rc::new(typ.forget()),
            dtors: dtors.clone(),
            impl_block: impl_block.forget(),
        }
    }
}

impl Forget for elab::Impl {
    type Target = ast::Impl;

    fn forget(&self) -> Self::Target {
        let elab::Impl { info, name, defs } = self;

        ast::Impl { info: info.forget(), name: name.clone(), defs: defs.clone() }
    }
}

impl Forget for elab::TypAbs {
    type Target = ast::TypAbs;

    fn forget(&self) -> Self::Target {
        let elab::TypAbs { params } = self;

        ast::TypAbs { params: params.forget() }
    }
}

impl Forget for elab::Ctor {
    type Target = ast::Ctor;

    fn forget(&self) -> Self::Target {
        let elab::Ctor { info, name, params, typ } = self;

        ast::Ctor {
            info: info.forget(),
            name: name.clone(),
            params: params.forget(),
            typ: typ.forget(),
        }
    }
}

impl Forget for elab::Dtor {
    type Target = ast::Dtor;

    fn forget(&self) -> Self::Target {
        let elab::Dtor { info, name, params, on_typ, in_typ } = self;

        ast::Dtor {
            info: info.forget(),
            name: name.clone(),
            params: params.forget(),
            on_typ: on_typ.forget(),
            in_typ: in_typ.forget(),
        }
    }
}

impl Forget for elab::Def {
    type Target = ast::Def;

    fn forget(&self) -> Self::Target {
        let elab::Def { info, name, params, on_typ, in_typ, body } = self;

        ast::Def {
            info: info.forget(),
            name: name.clone(),
            params: params.forget(),
            on_typ: on_typ.forget(),
            in_typ: in_typ.forget(),
            body: body.forget(),
        }
    }
}

impl Forget for elab::Codef {
    type Target = ast::Codef;

    fn forget(&self) -> Self::Target {
        let elab::Codef { info, name, params, typ, body } = self;

        ast::Codef {
            info: info.forget(),
            name: name.clone(),
            params: params.forget(),
            typ: typ.forget(),
            body: body.forget(),
        }
    }
}

impl Forget for elab::Match {
    type Target = ast::Match;

    fn forget(&self) -> Self::Target {
        let elab::Match { info, cases } = self;

        ast::Match { info: info.forget(), cases: cases.forget() }
    }
}

impl Forget for elab::Comatch {
    type Target = ast::Comatch;

    fn forget(&self) -> Self::Target {
        let elab::Comatch { info, cases } = self;

        ast::Comatch { info: info.forget(), cases: cases.forget() }
    }
}

impl Forget for elab::Case {
    type Target = ast::Case;

    fn forget(&self) -> Self::Target {
        let elab::Case { info, name, args, eqns, body } = self;

        ast::Case {
            info: info.forget(),
            name: name.clone(),
            args: args.forget(),
            eqns: eqns.forget(),
            body: body.forget(),
        }
    }
}

impl Forget for elab::Cocase {
    type Target = ast::Cocase;

    fn forget(&self) -> Self::Target {
        let elab::Cocase { info, name, args, eqns, body } = self;

        ast::Cocase {
            info: info.forget(),
            name: name.clone(),
            args: args.forget(),
            eqns: eqns.forget(),
            body: body.forget(),
        }
    }
}

impl Forget for elab::Eqn {
    type Target = ast::Eqn;

    fn forget(&self) -> Self::Target {
        let elab::Eqn { info, lhs, rhs } = self;

        ast::Eqn { info: info.forget(), lhs: lhs.forget(), rhs: rhs.forget() }
    }
}

impl Forget for elab::TypApp {
    type Target = ast::TypApp;

    fn forget(&self) -> Self::Target {
        let elab::TypApp { info, name, args } = self;

        ast::TypApp { info: info.forget(), name: name.clone(), args: args.forget() }
    }
}

impl Forget for elab::Exp {
    type Target = ast::Exp;

    fn forget(&self) -> Self::Target {
        match self {
            elab::Exp::Var { info, name, idx } => {
                ast::Exp::Var { info: info.forget(), name: name.clone(), idx: *idx }
            }
            elab::Exp::TyCtor { info, name, args } => {
                ast::Exp::TypCtor { info: info.forget(), name: name.clone(), args: args.forget() }
            }
            elab::Exp::Ctor { info, name, args } => {
                ast::Exp::Ctor { info: info.forget(), name: name.clone(), args: args.forget() }
            }
            elab::Exp::Dtor { info, exp, name, args } => ast::Exp::Dtor {
                info: info.forget(),
                exp: exp.forget(),
                name: name.clone(),
                args: args.forget(),
            },
            elab::Exp::Anno { info, exp, typ } => {
                ast::Exp::Anno { info: info.forget(), exp: exp.forget(), typ: typ.forget() }
            }
            elab::Exp::Type { info } => ast::Exp::Type { info: info.forget() },
        }
    }
}

impl Forget for elab::Telescope {
    type Target = ast::Telescope;

    fn forget(&self) -> Self::Target {
        let elab::Telescope(params) = self;

        ast::Telescope(params.forget())
    }
}

impl Forget for elab::Param {
    type Target = ast::Param;

    fn forget(&self) -> Self::Target {
        let elab::Param { name, typ } = self;

        ast::Param { name: name.clone(), typ: typ.forget() }
    }
}

impl Forget for elab::Eqns {
    type Target = Vec<ast::EqnParam>;

    fn forget(&self) -> Self::Target {
        let elab::Eqns { params, .. } = self;

        params.forget()
    }
}

impl Forget for elab::EqnParam {
    type Target = ast::EqnParam;

    fn forget(&self) -> Self::Target {
        let elab::EqnParam { name, eqn } = self;

        ast::EqnParam { name: name.clone(), eqn: eqn.forget() }
    }
}

impl Forget for elab::Info {
    type Target = ast::Info;

    fn forget(&self) -> Self::Target {
        let elab::Info { span } = self;

        ast::Info { span: *span }
    }
}

impl Forget for elab::TypedInfo {
    type Target = ast::Info;

    fn forget(&self) -> Self::Target {
        let elab::TypedInfo { span, .. } = self;

        ast::Info { span: *span }
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
