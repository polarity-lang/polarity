use std::rc::Rc;

use codespan::Span;
use syntax::ctx::*;
use syntax::generic::*;

use super::ctx::*;

// Renaming
//
// The AST representation primarily uses nameless DeBruijn indizes and levels to track
// the binding structure between binding and bound occurrences of variables.
// If we want to print a human-readable representation of the AST that can be
// parsed again, then we have to invent new names for the nameless variables which
// reflect the same binding structure: The creation of these new names is called "renaming".
//
// Example:
// The nameless representation of the const function which returns the first of two arguments
// is "\_ => \_ => 1"; renaming makes up variable names "x" and "y" to obtain the renamed term
// "\x => \y => x".
//
// Implementation:
//
// We traverse the AST while maintaining a context of variable names that are bound.
// Every time we come across a binding occurrence we check whether the name that is currently
// annotated is already bound in the context. If it isn't bound then we leave it unchanged,
// otherwise we choose a new name which is not already bound in the context.
// Every time we encounter a variable we look up the name in the context.

pub trait Rename: Sized {
    /// Assigns consistent names to all binding and bound variable occurrences.
    /// Should only be called on closed expressions or declarations.
    fn rename(self) -> Self {
        let mut ctx = Ctx::empty();
        self.rename_in_ctx(&mut ctx)
    }
    /// Assigns consistent names to all binding and bound variable occurrences.
    /// The provided `ctx` must contain names for all free variables of `self`.
    fn rename_in_ctx(self, ctx: &mut Ctx) -> Self;
}

impl<R: Rename + Clone> Rename for Rc<R> {
    fn rename_in_ctx(self, ctx: &mut Ctx) -> Self {
        let x = (*self).clone();
        Rc::new(x.rename_in_ctx(ctx))
    }
}

impl<P: Phase> Rename for Prg<P>
where
    P::TypeInfo: Rename,
    P::TypeAppInfo: Rename,
    P::Ctx: Rename,
    P::InfTyp: Rename,
{
    fn rename_in_ctx(self, ctx: &mut Ctx) -> Self {
        let Prg { decls } = self;

        Prg { decls: decls.rename_in_ctx(ctx) }
    }
}

impl<P: Phase> Rename for Decls<P>
where
    P::TypeInfo: Rename,
    P::TypeAppInfo: Rename,
    P::Ctx: Rename,
    P::InfTyp: Rename,
{
    fn rename_in_ctx(self, ctx: &mut Ctx) -> Self {
        Decls {
            map: self.map.into_iter().map(|(name, decl)| (name, decl.rename_in_ctx(ctx))).collect(),
            lookup_table: self.lookup_table,
        }
    }
}

impl<P: Phase> Rename for Decl<P>
where
    P::TypeInfo: Rename,
    P::TypeAppInfo: Rename,
    P::Ctx: Rename,
    P::InfTyp: Rename,
{
    fn rename_in_ctx(self, ctx: &mut Ctx) -> Self {
        match self {
            Decl::Data(data) => Decl::Data(data.rename_in_ctx(ctx)),
            Decl::Codata(codata) => Decl::Codata(codata.rename_in_ctx(ctx)),
            Decl::Ctor(ctor) => Decl::Ctor(ctor.rename_in_ctx(ctx)),
            Decl::Dtor(dtor) => Decl::Dtor(dtor.rename_in_ctx(ctx)),
            Decl::Def(def) => Decl::Def(def.rename_in_ctx(ctx)),
            Decl::Codef(codef) => Decl::Codef(codef.rename_in_ctx(ctx)),
            Decl::Let(lets) => Decl::Let(lets.rename_in_ctx(ctx)),
        }
    }
}

impl<P: Phase> Rename for Data<P>
where
    P::TypeInfo: Rename,
    P::TypeAppInfo: Rename,
    P::Ctx: Rename,
    P::InfTyp: Rename,
{
    fn rename_in_ctx(self, ctx: &mut Ctx) -> Self {
        let Data { info, doc, name, attr, typ, ctors } = self;
        Data { info, doc, name, attr, typ: typ.rename_in_ctx(ctx), ctors }
    }
}

impl<P: Phase> Rename for Codata<P>
where
    P::TypeInfo: Rename,
    P::TypeAppInfo: Rename,
    P::Ctx: Rename,
    P::InfTyp: Rename,
{
    fn rename_in_ctx(self, ctx: &mut Ctx) -> Self {
        let Codata { info, doc, name, attr, typ, dtors } = self;

        Codata { info, doc, name, attr, typ: typ.rename_in_ctx(ctx), dtors }
    }
}

impl<P: Phase> Rename for Ctor<P>
where
    P::TypeInfo: Rename,
    P::TypeAppInfo: Rename,
    P::Ctx: Rename,
    P::InfTyp: Rename,
{
    fn rename_in_ctx(self, ctx: &mut Ctx) -> Self {
        let Ctor { info, doc, name, params, typ } = self;
        let new_params = params.rename_in_ctx(ctx);
        let new_typ = ctx
            .bind_iter(new_params.params.clone().into_iter(), |new_ctx| typ.rename_in_ctx(new_ctx));

        Ctor { info, doc, name, params: new_params, typ: new_typ }
    }
}

impl<P: Phase> Rename for Dtor<P>
where
    P::TypeInfo: Rename,
    P::TypeAppInfo: Rename,
    P::Ctx: Rename,
    P::InfTyp: Rename,
{
    fn rename_in_ctx(self, ctx: &mut Ctx) -> Self {
        let Dtor { info, doc, name, params, self_param, ret_typ } = self;

        let new_params = params.rename_in_ctx(ctx);
        ctx.bind_iter(new_params.params.clone().into_iter(), |new_ctx| {
            let new_self = self_param.rename_in_ctx(new_ctx);

            new_ctx.bind_single(new_self.clone(), |new_ctx| {
                let new_ret = ret_typ.rename_in_ctx(new_ctx);
                Dtor { info, doc, name, params: new_params, self_param: new_self, ret_typ: new_ret }
            })
        })
    }
}

impl<P: Phase> Rename for Def<P>
where
    P::TypeInfo: Rename,
    P::TypeAppInfo: Rename,
    P::Ctx: Rename,
    P::InfTyp: Rename,
{
    fn rename_in_ctx(self, ctx: &mut Ctx) -> Self {
        let Def { info, doc, name, attr, params, self_param, ret_typ, body } = self;

        let new_params = params.rename_in_ctx(ctx);
        ctx.bind_iter(new_params.params.clone().into_iter(), |new_ctx| {
            let new_self = self_param.rename_in_ctx(new_ctx);
            let new_body = body.rename_in_ctx(new_ctx);

            new_ctx.bind_single(new_self.clone(), |new_ctx| {
                let new_ret = ret_typ.rename_in_ctx(new_ctx);
                Def {
                    info,
                    doc,
                    name,
                    attr,
                    params: new_params,
                    self_param: new_self,
                    ret_typ: new_ret,
                    body: new_body,
                }
            })
        })
    }
}

impl<P: Phase> Rename for Codef<P>
where
    P::TypeInfo: Rename,
    P::TypeAppInfo: Rename,
    P::Ctx: Rename,
    P::InfTyp: Rename,
{
    fn rename_in_ctx(self, ctx: &mut Ctx) -> Self {
        let Codef { info, doc, name, attr, params, typ, body } = self;

        let new_params = params.rename_in_ctx(ctx);

        ctx.bind_iter(new_params.params.clone().into_iter(), |new_ctx| {
            let new_typ = typ.rename_in_ctx(new_ctx);

            let new_body = body.rename_in_ctx(new_ctx);

            Codef { info, doc, name, attr, params: new_params, typ: new_typ, body: new_body }
        })
    }
}

impl<P: Phase> Rename for Let<P>
where
    P::TypeInfo: Rename,
    P::TypeAppInfo: Rename,
    P::Ctx: Rename,
    P::InfTyp: Rename,
{
    fn rename_in_ctx(self, ctx: &mut Ctx) -> Self {
        let Let { info, doc, name, attr, params, typ, body } = self;

        let new_params = params.rename_in_ctx(ctx);

        ctx.bind_iter(new_params.params.clone().into_iter(), |new_ctx| {
            let new_typ = typ.rename_in_ctx(new_ctx);

            let new_body = body.rename_in_ctx(new_ctx);

            Let { info, doc, name, attr, params: new_params, typ: new_typ, body: new_body }
        })
    }
}

impl<P: Phase> Rename for TypAbs<P>
where
    P::TypeInfo: Rename,
    P::TypeAppInfo: Rename,
    P::Ctx: Rename,
    P::InfTyp: Rename,
{
    fn rename_in_ctx(self, ctx: &mut Ctx) -> Self {
        let TypAbs { params } = self;
        TypAbs { params: params.rename_in_ctx(ctx) }
    }
}

impl<P: Phase> Rename for Telescope<P>
where
    P::TypeInfo: Rename,
    P::TypeAppInfo: Rename,
    P::Ctx: Rename,
    P::InfTyp: Rename,
{
    fn rename_in_ctx(self, ctx: &mut Ctx) -> Self {
        let Telescope { params } = self;

        ctx.bind_fold2(
            params.into_iter(),
            vec![],
            |ctx, mut acc, mut param| {
                param = param.rename_in_ctx(ctx);
                let new_name = param.name.clone();
                acc.push(param);
                BindElem { elem: new_name, ret: acc }
            },
            |_ctx, params| Telescope { params },
        )
    }
}

impl<P: Phase> Rename for Param<P>
where
    P::TypeInfo: Rename,
    P::TypeAppInfo: Rename,
    P::Ctx: Rename,
    P::InfTyp: Rename,
{
    fn rename_in_ctx(self, ctx: &mut Ctx) -> Self {
        let Param { name, typ } = self;

        let new_typ = typ.rename_in_ctx(ctx);
        let new_name = ctx.disambiguate_name(name);

        Param { name: new_name, typ: new_typ }
    }
}

impl<P: Phase> Rename for TelescopeInst<P>
where
    P::TypeInfo: Rename,
    P::TypeAppInfo: Rename,
    P::Ctx: Rename,
    P::InfTyp: Rename,
{
    fn rename_in_ctx(self, ctx: &mut Ctx) -> Self {
        let TelescopeInst { params } = self;

        ctx.bind_fold2(
            params.into_iter(),
            vec![],
            |ctx, mut acc, mut param| {
                param = param.rename_in_ctx(ctx);
                let new_name = param.name.clone();
                acc.push(param);
                BindElem { elem: new_name, ret: acc }
            },
            |_ctx, params| TelescopeInst { params },
        )
    }
}

impl<P: Phase> Rename for ParamInst<P>
where
    P::TypeInfo: Rename,
    P::TypeAppInfo: Rename,
    P::Ctx: Rename,
    P::InfTyp: Rename,
{
    fn rename_in_ctx(self, ctx: &mut Ctx) -> Self {
        let ParamInst { name, typ, info } = self;

        let new_typ = typ.rename_in_ctx(ctx);
        let new_name = ctx.disambiguate_name(name);
        let new_info = info.rename_in_ctx(ctx);

        ParamInst { name: new_name, typ: new_typ, info: new_info }
    }
}
impl<P: Phase> Rename for TypApp<P>
where
    P::TypeInfo: Rename,
    P::TypeAppInfo: Rename,
    P::Ctx: Rename,
    P::InfTyp: Rename,
{
    fn rename_in_ctx(self, ctx: &mut Ctx) -> Self {
        let TypApp { info, name, args } = self;

        TypApp { info: info.rename_in_ctx(ctx), name, args: args.rename_in_ctx(ctx) }
    }
}

impl<P: Phase> Rename for SelfParam<P>
where
    P::TypeInfo: Rename,
    P::TypeAppInfo: Rename,
    P::Ctx: Rename,
    P::InfTyp: Rename,
{
    fn rename_in_ctx(self, ctx: &mut Ctx) -> Self {
        let SelfParam { info, name, typ } = self;

        let new_name = name.map(|name| ctx.disambiguate_name(name));

        SelfParam { info, name: new_name, typ: typ.rename_in_ctx(ctx) }
    }
}

impl<P: Phase> Rename for Exp<P>
where
    P::TypeInfo: Rename,
    P::TypeAppInfo: Rename,
    P::Ctx: Rename,
    P::InfTyp: Rename,
{
    fn rename_in_ctx(self, ctx: &mut Ctx) -> Self {
        match self {
            Exp::Var { info, name: _, ctx: ctx2, idx } => {
                // This is the only place where we look up the renamed variables from the context
                let ctx2 = ctx2.rename_in_ctx(ctx);
                Exp::Var { info: info.rename_in_ctx(ctx), name: ctx.lookup(idx), ctx: ctx2, idx }
            }
            Exp::LocalComatch { info, ctx: ctx2, name, is_lambda_sugar, body } => {
                Exp::LocalComatch {
                    info: info.rename_in_ctx(ctx),
                    ctx: ctx2.rename_in_ctx(ctx),
                    name,
                    is_lambda_sugar,
                    body: body.rename_in_ctx(ctx),
                }
            }
            Exp::Anno { info, exp, typ } => Exp::Anno {
                info: info.rename_in_ctx(ctx),
                exp: exp.rename_in_ctx(ctx),
                typ: typ.rename_in_ctx(ctx),
            },
            Exp::TypCtor { info, name, args } => {
                Exp::TypCtor { info: info.rename_in_ctx(ctx), name, args: args.rename_in_ctx(ctx) }
            }
            Exp::Hole { info } => Exp::Hole { info: info.rename_in_ctx(ctx) },
            Exp::Type { info } => Exp::Type { info: info.rename_in_ctx(ctx) },
            Exp::Ctor { info, name, args } => {
                Exp::Ctor { info: info.rename_in_ctx(ctx), name, args: args.rename_in_ctx(ctx) }
            }
            Exp::LocalMatch { info, ctx: ctx2, name, on_exp, motive, ret_typ, body } => {
                Exp::LocalMatch {
                    info: info.rename_in_ctx(ctx),
                    ctx: ctx2.rename_in_ctx(ctx),
                    name,
                    on_exp: on_exp.rename_in_ctx(ctx),
                    motive: motive.rename_in_ctx(ctx),
                    ret_typ: ret_typ.rename_in_ctx(ctx),
                    body: body.rename_in_ctx(ctx),
                }
            }
            Exp::Dtor { info, exp, name, args } => Exp::Dtor {
                info: info.rename_in_ctx(ctx),
                name,
                exp: exp.rename_in_ctx(ctx),
                args: args.rename_in_ctx(ctx),
            },
        }
    }
}

impl<P: Phase> Rename for Match<P>
where
    P::TypeInfo: Rename,
    P::TypeAppInfo: Rename,
    P::Ctx: Rename,
    P::InfTyp: Rename,
{
    fn rename_in_ctx(self, ctx: &mut Ctx) -> Self {
        let Match { info, cases, omit_absurd } = self;

        Match { info, cases: cases.rename_in_ctx(ctx), omit_absurd }
    }
}

impl<P: Phase> Rename for Args<P>
where
    P::TypeInfo: Rename,
    P::TypeAppInfo: Rename,
    P::Ctx: Rename,
    P::InfTyp: Rename,
{
    fn rename_in_ctx(self, ctx: &mut Ctx) -> Self {
        let Args { args } = self;

        Args { args: args.rename_in_ctx(ctx) }
    }
}

impl<P: Phase> Rename for Case<P>
where
    P::TypeInfo: Rename,
    P::TypeAppInfo: Rename,
    P::Ctx: Rename,
    P::InfTyp: Rename,
{
    fn rename_in_ctx(self, ctx: &mut Ctx) -> Self {
        let Case { info, name, args, body } = self;

        let new_args = args.rename_in_ctx(ctx);

        ctx.bind_iter(new_args.params.clone().into_iter(), |new_ctx| {
            let new_body = body.rename_in_ctx(new_ctx);

            Case { info, name, args: new_args, body: new_body }
        })
    }
}

//     fn map_motive_param<X, F>(&mut self, mut param: ParamInst<P>, f_inner: F) -> X
//     where
//         F: FnOnce(&mut Self, ParamInst<P>) -> X,
//     {
//         param.name = self.disambiguate_name(param.name);
//         self.ctx_map_motive_param(param, f_inner)
//     }

impl<P: Phase> Rename for Motive<P>
where
    P::TypeInfo: Rename,
    P::TypeAppInfo: Rename,
    P::Ctx: Rename,
    P::InfTyp: Rename,
{
    fn rename_in_ctx(self, ctx: &mut Ctx) -> Self {
        let Motive { info, param, ret_typ } = self;

        let new_param = param.rename_in_ctx(ctx);
        ctx.bind_single(new_param.clone(), |new_ctx| {
            let new_ret_typ = ret_typ.rename_in_ctx(new_ctx);

            Motive { info, param: new_param, ret_typ: new_ret_typ }
        })
    }
}

impl Rename for Option<Span> {
    fn rename_in_ctx(self, _ctx: &mut crate::Ctx) -> Self {
        self
    }
}

impl<T: Rename> Rename for Option<T> {
    fn rename_in_ctx(self, ctx: &mut Ctx) -> Self {
        self.map(|x| x.rename_in_ctx(ctx))
    }
}

impl<T: Rename> Rename for Vec<T> {
    fn rename_in_ctx(self, ctx: &mut Ctx) -> Self {
        self.into_iter().map(|x| x.rename_in_ctx(ctx)).collect()
    }
}

impl Rename for () {
    fn rename_in_ctx(self, _ctx: &mut Ctx) -> Self {}
}
