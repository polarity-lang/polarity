use std::rc::Rc;

use syntax::ast::*;

use super::ctx::Ctx;
use super::{Rename, RenameTelescope};

impl Rename for Prg {
    fn rename_in_ctx(&self, ctx: &mut Ctx) -> Self {
        let Prg { decls, exp } = self;
        Prg { decls: decls.rename_in_ctx(ctx), exp: exp.rename_in_ctx(ctx) }
    }
}

impl Rename for Decls {
    fn rename_in_ctx(&self, ctx: &mut Ctx) -> Self {
        let Decls { map, order } = self;
        let map = map.iter().map(|(name, decl)| (name.clone(), decl.rename_in_ctx(ctx))).collect();
        Decls { map, order: order.clone() }
    }
}

impl Rename for Decl {
    fn rename_in_ctx(&self, ctx: &mut Ctx) -> Self {
        match self {
            Decl::Data(data) => Decl::Data(data.rename_in_ctx(ctx)),
            Decl::Codata(codata) => Decl::Codata(codata.rename_in_ctx(ctx)),
            Decl::Ctor(ctor) => Decl::Ctor(ctor.rename_in_ctx(ctx)),
            Decl::Dtor(dtor) => Decl::Dtor(dtor.rename_in_ctx(ctx)),
            Decl::Def(def) => Decl::Def(def.rename_in_ctx(ctx)),
            Decl::Codef(codef) => Decl::Codef(codef.rename_in_ctx(ctx)),
        }
    }
}

impl Rename for Data {
    fn rename_in_ctx(&self, ctx: &mut Ctx) -> Self {
        let Data { info, name, typ, ctors, impl_block } = self;
        Data {
            info: info.clone(),
            name: name.clone(),
            typ: typ.rename_in_ctx(ctx),
            ctors: ctors.clone(),
            impl_block: impl_block.rename_in_ctx(ctx),
        }
    }
}

impl Rename for Codata {
    fn rename_in_ctx(&self, ctx: &mut Ctx) -> Self {
        let Codata { info, name, typ, dtors, impl_block } = self;
        Codata {
            info: info.clone(),
            name: name.clone(),
            typ: typ.rename_in_ctx(ctx),
            dtors: dtors.clone(),
            impl_block: impl_block.rename_in_ctx(ctx),
        }
    }
}

impl Rename for Impl {
    fn rename_in_ctx(&self, _ctx: &mut Ctx) -> Self {
        let Impl { info, name, defs } = self;
        Impl { info: info.clone(), name: name.clone(), defs: defs.clone() }
    }
}

impl Rename for TypAbs {
    fn rename_in_ctx(&self, ctx: &mut Ctx) -> Self {
        let TypAbs { params } = self;
        TypAbs { params: params.rename_telescope(ctx, |_, out| out) }
    }
}

impl Rename for Ctor {
    fn rename_in_ctx(&self, ctx: &mut Ctx) -> Self {
        let Ctor { info, name, params, typ } = self;
        params.rename_telescope(ctx, |ctx, params| Ctor {
            info: info.clone(),
            name: name.clone(),
            params,
            typ: typ.rename_in_ctx(ctx),
        })
    }
}

impl Rename for Dtor {
    fn rename_in_ctx(&self, ctx: &mut Ctx) -> Self {
        let Dtor { info, name, params, on_typ, in_typ } = self;
        params.rename_telescope(ctx, |ctx, params| Dtor {
            info: info.clone(),
            name: name.clone(),
            params,
            on_typ: on_typ.rename_in_ctx(ctx),
            in_typ: in_typ.rename_in_ctx(ctx),
        })
    }
}

impl Rename for Def {
    fn rename_in_ctx(&self, ctx: &mut Ctx) -> Self {
        let Def { info, name, params, on_typ, in_typ, body } = self;
        params.rename_telescope(ctx, |ctx, params| Def {
            info: info.clone(),
            name: name.clone(),
            params,
            on_typ: on_typ.rename_in_ctx(ctx),
            in_typ: in_typ.rename_in_ctx(ctx),
            body: body.rename_in_ctx(ctx),
        })
    }
}

impl Rename for Codef {
    fn rename_in_ctx(&self, ctx: &mut Ctx) -> Self {
        let Codef { info, name, params, typ, body } = self;
        params.rename_telescope(ctx, |ctx, params| Codef {
            info: info.clone(),
            name: name.clone(),
            params,
            typ: typ.rename_in_ctx(ctx),
            body: body.rename_in_ctx(ctx),
        })
    }
}

impl Rename for Match {
    fn rename_in_ctx(&self, ctx: &mut Ctx) -> Self {
        let Match { info, cases } = self;
        Match { info: info.clone(), cases: cases.rename_in_ctx(ctx) }
    }
}

impl Rename for Comatch {
    fn rename_in_ctx(&self, ctx: &mut Ctx) -> Self {
        let Comatch { info, cases } = self;
        Comatch { info: info.clone(), cases: cases.rename_in_ctx(ctx) }
    }
}

impl Rename for Case {
    fn rename_in_ctx(&self, ctx: &mut Ctx) -> Self {
        let Case { info, name, args, eqns, body } = self;
        args.rename_telescope(ctx, |ctx, args| {
            eqns.rename_telescope(ctx, |ctx, eqns| Case {
                info: info.clone(),
                name: name.clone(),
                args,
                eqns,
                body: body.rename_in_ctx(ctx),
            })
        })
    }
}

impl Rename for Cocase {
    fn rename_in_ctx(&self, ctx: &mut Ctx) -> Self {
        let Cocase { info, name, args, eqns, body } = self;
        args.rename_telescope(ctx, |ctx, args| {
            eqns.rename_telescope(ctx, |ctx, eqns| Cocase {
                info: info.clone(),
                name: name.clone(),
                args,
                eqns,
                body: body.rename_in_ctx(ctx),
            })
        })
    }
}

impl Rename for Eqn {
    fn rename_in_ctx(&self, ctx: &mut Ctx) -> Self {
        let Eqn { info, lhs, rhs } = self;
        Eqn { info: info.clone(), lhs: lhs.rename_in_ctx(ctx), rhs: rhs.rename_in_ctx(ctx) }
    }
}

impl Rename for TypApp {
    fn rename_in_ctx(&self, ctx: &mut Ctx) -> Self {
        let TypApp { info, name, args } = self;
        TypApp { info: info.clone(), name: name.clone(), args: args.rename_in_ctx(ctx) }
    }
}

impl Rename for Exp {
    fn rename_in_ctx(&self, ctx: &mut Ctx) -> Self {
        use Exp::*;
        match self {
            Var { info, name: _, idx } => {
                Var { info: info.clone(), name: ctx.bound(*idx), idx: *idx }
            }
            TypCtor { info, name, args } => {
                TypCtor { info: info.clone(), name: name.clone(), args: args.rename_in_ctx(ctx) }
            }
            Ctor { info, name, args } => {
                Ctor { info: info.clone(), name: name.clone(), args: args.rename_in_ctx(ctx) }
            }
            Dtor { info, exp, name, args } => Dtor {
                info: info.clone(),
                exp: exp.rename_in_ctx(ctx),
                name: name.clone(),
                args: args.rename_in_ctx(ctx),
            },
            Anno { info, exp, typ } => Anno {
                info: info.clone(),
                exp: exp.rename_in_ctx(ctx),
                typ: typ.rename_in_ctx(ctx),
            },
            Type { info } => Type { info: info.clone() },
        }
    }
}

impl Rename for Param {
    fn rename_in_ctx(&self, ctx: &mut Ctx) -> Self {
        let Param { name, typ } = self;
        Param { name: name.clone(), typ: typ.rename_in_ctx(ctx) }
    }
}

impl Rename for EqnParam {
    fn rename_in_ctx(&self, ctx: &mut Ctx) -> Self {
        let EqnParam { name, eqn } = self;
        EqnParam { name: name.clone(), eqn: eqn.rename_in_ctx(ctx) }
    }
}

impl<T: Rename> Rename for Option<T> {
    fn rename_in_ctx(&self, ctx: &mut Ctx) -> Self {
        self.as_ref().map(|x| x.rename_in_ctx(ctx))
    }
}

impl<T: Rename> Rename for Rc<T> {
    fn rename_in_ctx(&self, ctx: &mut Ctx) -> Self {
        Rc::new(T::rename_in_ctx(self, ctx))
    }
}

impl RenameTelescope for Telescope {
    fn rename_telescope<T, F: FnOnce(&mut Ctx, Self) -> T>(&self, ctx: &mut Ctx, f: F) -> T {
        let Telescope(params) = self;
        ctx.bind_fold(
            params.iter(),
            vec![],
            |ctx, mut params_out, param| {
                params_out.push(param.rename_in_ctx(ctx));
                params_out
            },
            |ctx, params_out| f(ctx, Telescope(params_out)),
        )
    }
}

impl RenameTelescope for EqnParams {
    fn rename_telescope<T, F: FnOnce(&mut Ctx, Self) -> T>(&self, ctx: &mut Ctx, f: F) -> T {
        ctx.bind_fold(
            self.iter(),
            vec![],
            |ctx, mut params_out, param| {
                params_out.push(param.rename_in_ctx(ctx));
                params_out
            },
            f,
        )
    }
}

impl<T: Rename> Rename for Vec<T> {
    fn rename_in_ctx(&self, ctx: &mut crate::ctx::Ctx) -> Self {
        self.iter().map(|x| x.rename_in_ctx(ctx)).collect()
    }
}
