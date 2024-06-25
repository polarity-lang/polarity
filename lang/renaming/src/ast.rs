use std::rc::Rc;

use codespan::Span;
use syntax::ast::*;
use syntax::ctx::*;

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

impl Rename for Module {
    fn rename_in_ctx(self, ctx: &mut Ctx) -> Self {
        Module {
            uri: self.uri,
            map: self.map.into_iter().map(|(name, decl)| (name, decl.rename_in_ctx(ctx))).collect(),
            lookup_table: self.lookup_table,
            meta_vars: self.meta_vars,
        }
    }
}

impl Rename for Decl {
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

impl Rename for Data {
    fn rename_in_ctx(self, ctx: &mut Ctx) -> Self {
        let Data { span, doc, name, attr, typ, ctors } = self;
        Data { span, doc, name, attr, typ: typ.rename_in_ctx(ctx), ctors }
    }
}

impl Rename for Codata {
    fn rename_in_ctx(self, ctx: &mut Ctx) -> Self {
        let Codata { span, doc, name, attr, typ, dtors } = self;

        Codata { span, doc, name, attr, typ: typ.rename_in_ctx(ctx), dtors }
    }
}

impl Rename for Ctor {
    fn rename_in_ctx(self, ctx: &mut Ctx) -> Self {
        let Ctor { span, doc, name, params, typ } = self;
        let new_params = params.rename_in_ctx(ctx);
        let new_typ = ctx
            .bind_iter(new_params.params.clone().into_iter(), |new_ctx| typ.rename_in_ctx(new_ctx));

        Ctor { span, doc, name, params: new_params, typ: new_typ }
    }
}

impl Rename for Dtor {
    fn rename_in_ctx(self, ctx: &mut Ctx) -> Self {
        let Dtor { span, doc, name, params, self_param, ret_typ } = self;

        let new_params = params.rename_in_ctx(ctx);
        ctx.bind_iter(new_params.params.clone().into_iter(), |new_ctx| {
            let new_self = self_param.rename_in_ctx(new_ctx);

            new_ctx.bind_single(new_self.clone(), |new_ctx| {
                let new_ret = ret_typ.rename_in_ctx(new_ctx);
                Dtor { span, doc, name, params: new_params, self_param: new_self, ret_typ: new_ret }
            })
        })
    }
}

impl Rename for Def {
    fn rename_in_ctx(self, ctx: &mut Ctx) -> Self {
        let Def { span, doc, name, attr, params, self_param, ret_typ, cases } = self;

        let new_params = params.rename_in_ctx(ctx);
        ctx.bind_iter(new_params.params.clone().into_iter(), |new_ctx| {
            let new_self = self_param.rename_in_ctx(new_ctx);
            let new_cases = cases.rename_in_ctx(new_ctx);

            new_ctx.bind_single(new_self.clone(), |new_ctx| {
                let new_ret = ret_typ.rename_in_ctx(new_ctx);
                Def {
                    span,
                    doc,
                    name,
                    attr,
                    params: new_params,
                    self_param: new_self,
                    ret_typ: new_ret,
                    cases: new_cases,
                }
            })
        })
    }
}

impl Rename for Codef {
    fn rename_in_ctx(self, ctx: &mut Ctx) -> Self {
        let Codef { span, doc, name, attr, params, typ, cases } = self;

        let new_params = params.rename_in_ctx(ctx);

        ctx.bind_iter(new_params.params.clone().into_iter(), |new_ctx| {
            let new_typ = typ.rename_in_ctx(new_ctx);

            let new_cases = cases.rename_in_ctx(new_ctx);

            Codef { span, doc, name, attr, params: new_params, typ: new_typ, cases: new_cases }
        })
    }
}

impl Rename for Let {
    fn rename_in_ctx(self, ctx: &mut Ctx) -> Self {
        let Let { span, doc, name, attr, params, typ, body } = self;

        let new_params = params.rename_in_ctx(ctx);

        ctx.bind_iter(new_params.params.clone().into_iter(), |new_ctx| {
            let new_typ = typ.rename_in_ctx(new_ctx);

            let new_body = body.rename_in_ctx(new_ctx);

            Let { span, doc, name, attr, params: new_params, typ: new_typ, body: new_body }
        })
    }
}

impl Rename for Telescope {
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

impl Rename for Param {
    fn rename_in_ctx(self, ctx: &mut Ctx) -> Self {
        let Param { implicit, name, typ } = self;

        let new_typ = typ.rename_in_ctx(ctx);
        let new_name = ctx.disambiguate_name(name);

        Param { implicit, name: new_name, typ: new_typ }
    }
}

impl Rename for TelescopeInst {
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

impl Rename for ParamInst {
    fn rename_in_ctx(self, ctx: &mut Ctx) -> Self {
        let ParamInst { span, name, typ, .. } = self;

        let new_typ = typ.rename_in_ctx(ctx);
        let new_name = ctx.disambiguate_name(name);

        ParamInst { span, name: new_name, typ: new_typ, info: None }
    }
}

impl Rename for SelfParam {
    fn rename_in_ctx(self, ctx: &mut Ctx) -> Self {
        let SelfParam { info, name, typ } = self;

        let new_name = name.map(|name| ctx.disambiguate_name(name));

        SelfParam { info, name: new_name, typ: typ.rename_in_ctx(ctx) }
    }
}

impl Rename for Exp {
    fn rename_in_ctx(self, ctx: &mut Ctx) -> Self {
        match self {
            Exp::Variable(e) => e.rename_in_ctx(ctx).into(),
            Exp::LocalComatch(e) => e.rename_in_ctx(ctx).into(),
            Exp::Anno(e) => e.rename_in_ctx(ctx).into(),
            Exp::TypCtor(e) => e.rename_in_ctx(ctx).into(),
            Exp::Hole(e) => e.rename_in_ctx(ctx).into(),
            Exp::TypeUniv(e) => e.rename_in_ctx(ctx).into(),
            Exp::Call(e) => e.rename_in_ctx(ctx).into(),
            Exp::LocalMatch(e) => e.rename_in_ctx(ctx).into(),
            Exp::DotCall(e) => e.rename_in_ctx(ctx).into(),
        }
    }
}

impl Rename for TypeUniv {
    fn rename_in_ctx(self, _ctx: &mut Ctx) -> Self {
        self
    }
}

impl Rename for Call {
    fn rename_in_ctx(self, ctx: &mut Ctx) -> Self {
        let Call { span, name, args, inferred_type, kind } = self;
        Call {
            span,
            kind,
            name,
            args: args.rename_in_ctx(ctx),
            inferred_type: inferred_type.rename_in_ctx(ctx),
        }
    }
}

impl Rename for DotCall {
    fn rename_in_ctx(self, ctx: &mut Ctx) -> Self {
        let DotCall { span, kind, exp, name, args, inferred_type } = self;
        DotCall {
            span,
            kind,
            name,
            exp: exp.rename_in_ctx(ctx),
            args: args.rename_in_ctx(ctx),
            inferred_type: inferred_type.rename_in_ctx(ctx),
        }
    }
}
impl Rename for Variable {
    fn rename_in_ctx(self, ctx: &mut Ctx) -> Self {
        let Variable { span, idx, inferred_type, .. } = self;
        Variable {
            span,
            idx,
            name: ctx.lookup(idx),
            inferred_type: inferred_type.rename_in_ctx(ctx),
        }
    }
}

impl Rename for LocalComatch {
    fn rename_in_ctx(self, ctx: &mut Ctx) -> Self {
        let LocalComatch { span, name, is_lambda_sugar, cases, .. } = self;
        LocalComatch {
            span,
            ctx: None,
            name,
            is_lambda_sugar,
            cases: cases.rename_in_ctx(ctx),
            inferred_type: None,
        }
    }
}

impl Rename for Hole {
    fn rename_in_ctx(self, ctx: &mut Ctx) -> Self {
        let Hole { span, metavar, inferred_type, inferred_ctx: _, args } = self;
        Hole {
            span,
            metavar,
            inferred_type: inferred_type.rename_in_ctx(ctx),
            inferred_ctx: None, // TODO: Rename TypeCtx!
            args: args.rename_in_ctx(ctx),
        }
    }
}
impl Rename for LocalMatch {
    fn rename_in_ctx(self, ctx: &mut Ctx) -> Self {
        let LocalMatch { span, name, on_exp, motive, ret_typ, cases, .. } = self;
        LocalMatch {
            span,
            ctx: None,
            name,
            on_exp: on_exp.rename_in_ctx(ctx),
            motive: motive.rename_in_ctx(ctx),
            ret_typ: ret_typ.rename_in_ctx(ctx),
            cases: cases.rename_in_ctx(ctx),
            inferred_type: None,
        }
    }
}

impl Rename for TypCtor {
    fn rename_in_ctx(self, ctx: &mut Ctx) -> Self {
        let TypCtor { span, name, args } = self;
        TypCtor { span, name, args: args.rename_in_ctx(ctx) }
    }
}

impl Rename for Anno {
    fn rename_in_ctx(self, ctx: &mut Ctx) -> Self {
        let Anno { span, exp, typ, normalized_type } = self;
        Anno {
            span,
            exp: exp.rename_in_ctx(ctx),
            typ: typ.rename_in_ctx(ctx),
            normalized_type: normalized_type.map(|e| e.rename_in_ctx(ctx)),
        }
    }
}

impl Rename for Args {
    fn rename_in_ctx(self, ctx: &mut Ctx) -> Self {
        let Args { args } = self;

        Args { args: args.rename_in_ctx(ctx) }
    }
}

impl Rename for Case {
    fn rename_in_ctx(self, ctx: &mut Ctx) -> Self {
        let Case { span, name, params, body } = self;

        let new_params = params.rename_in_ctx(ctx);

        ctx.bind_iter(new_params.params.clone().into_iter(), |new_ctx| {
            let new_body = body.rename_in_ctx(new_ctx);

            Case { span, name, params: new_params, body: new_body }
        })
    }
}

impl Rename for Motive {
    fn rename_in_ctx(self, ctx: &mut Ctx) -> Self {
        let Motive { span, param, ret_typ } = self;

        let new_param = param.rename_in_ctx(ctx);
        ctx.bind_single(new_param.clone(), |new_ctx| {
            let new_ret_typ = ret_typ.rename_in_ctx(new_ctx);

            Motive { span, param: new_param, ret_typ: new_ret_typ }
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
