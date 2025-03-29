use ast::ctx::*;
use ast::*;
use miette_util::codespan::Span;
use values::Binder;

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
    fn rename(&mut self) {
        let mut ctx = GenericCtx::empty().into();
        self.rename_in_ctx(&mut ctx)
    }
    /// Assigns consistent names to all binding and bound variable occurrences.
    /// The provided `ctx` must contain names for all free variables of `self`.
    fn rename_in_ctx(&mut self, ctx: &mut Ctx);
}

impl<R: Rename + Clone> Rename for Box<R> {
    fn rename_in_ctx(&mut self, ctx: &mut Ctx) {
        (**self).rename_in_ctx(ctx);
    }
}

impl Rename for Module {
    fn rename_in_ctx(&mut self, ctx: &mut Ctx) {
        self.decls.rename_in_ctx(ctx);
    }
}

impl Rename for Decl {
    fn rename_in_ctx(&mut self, ctx: &mut Ctx) {
        match self {
            Decl::Data(data) => data.rename_in_ctx(ctx),
            Decl::Codata(codata) => codata.rename_in_ctx(ctx),
            Decl::Def(def) => def.rename_in_ctx(ctx),
            Decl::Codef(codef) => codef.rename_in_ctx(ctx),
            Decl::Let(lets) => lets.rename_in_ctx(ctx),
            Decl::Infix(infix) => infix.rename_in_ctx(ctx),
        }
    }
}

impl Rename for Data {
    fn rename_in_ctx(&mut self, ctx: &mut Ctx) {
        self.ctors.rename_in_ctx(ctx);
        self.typ.rename_in_ctx(ctx);
    }
}

impl Rename for Codata {
    fn rename_in_ctx(&mut self, ctx: &mut Ctx) {
        self.dtors.rename_in_ctx(ctx);
        self.typ.rename_in_ctx(ctx);
    }
}

impl Rename for Ctor {
    fn rename_in_ctx(&mut self, ctx: &mut Ctx) {
        self.params.rename_in_ctx(ctx);
        ctx.bind_iter(self.params.params.iter(), |new_ctx| self.typ.rename_in_ctx(new_ctx));
    }
}

impl Rename for Dtor {
    fn rename_in_ctx(&mut self, ctx: &mut Ctx) {
        self.params.rename_in_ctx(ctx);
        ctx.bind_iter(self.params.params.iter(), |new_ctx| {
            self.self_param.rename_in_ctx(new_ctx);

            new_ctx.bind_single(&self.self_param, |new_ctx| {
                self.ret_typ.rename_in_ctx(new_ctx);
            })
        })
    }
}

impl Rename for Def {
    fn rename_in_ctx(&mut self, ctx: &mut Ctx) {
        self.params.rename_in_ctx(ctx);
        ctx.bind_iter(self.params.params.iter(), |new_ctx| {
            self.self_param.rename_in_ctx(new_ctx);
            self.cases.rename_in_ctx(new_ctx);

            new_ctx.bind_single(&self.self_param, |new_ctx| self.ret_typ.rename_in_ctx(new_ctx))
        })
    }
}

impl Rename for Codef {
    fn rename_in_ctx(&mut self, ctx: &mut Ctx) {
        self.params.rename_in_ctx(ctx);

        ctx.bind_iter(self.params.params.iter(), |new_ctx| {
            self.typ.rename_in_ctx(new_ctx);
            self.cases.rename_in_ctx(new_ctx);
        })
    }
}

impl Rename for Let {
    fn rename_in_ctx(&mut self, ctx: &mut Ctx) {
        self.params.rename_in_ctx(ctx);
        ctx.bind_iter(self.params.params.iter(), |new_ctx| {
            self.typ.rename_in_ctx(new_ctx);
            self.body.rename_in_ctx(new_ctx);
        })
    }
}

impl Rename for Telescope {
    fn rename_in_ctx(&mut self, ctx: &mut Ctx) {
        let Telescope { params } = self;
        ctx.bind_fold(
            params.iter_mut(),
            vec![],
            |ctx, acc, param| {
                param.rename_in_ctx(ctx);
                let new_name = param.name.clone();
                acc.push(param);
                Binder { name: new_name, content: () }
            },
            |_ctx, _params| (),
        )
    }
}

impl Rename for Param {
    fn rename_in_ctx(&mut self, ctx: &mut Ctx) {
        self.typ.rename_in_ctx(ctx);
        self.name = ctx.disambiguate_var_bind(self.name.clone());
    }
}

impl Rename for TelescopeInst {
    fn rename_in_ctx(&mut self, ctx: &mut Ctx) {
        let TelescopeInst { params } = self;

        ctx.bind_fold(
            params.iter_mut(),
            vec![],
            |ctx, acc, param| {
                param.rename_in_ctx(ctx);
                let new_name = param.name.clone();
                acc.push(param);
                Binder { name: new_name, content: () }
            },
            |_ctx, _params| (),
        )
    }
}

impl Rename for ParamInst {
    fn rename_in_ctx(&mut self, ctx: &mut Ctx) {
        self.typ.rename_in_ctx(ctx);
        self.name = ctx.disambiguate_var_bind(self.name.clone());
    }
}

impl Rename for SelfParam {
    fn rename_in_ctx(&mut self, ctx: &mut Ctx) {
        let new_name = ctx.disambiguate_var_bind(self.name.clone());
        self.name = new_name;
        self.typ.rename_in_ctx(ctx);
    }
}

impl Rename for Exp {
    fn rename_in_ctx(&mut self, ctx: &mut Ctx) {
        match self {
            Exp::Variable(e) => e.rename_in_ctx(ctx),
            Exp::LocalComatch(e) => e.rename_in_ctx(ctx),
            Exp::Anno(e) => e.rename_in_ctx(ctx),
            Exp::TypCtor(e) => e.rename_in_ctx(ctx),
            Exp::Hole(e) => e.rename_in_ctx(ctx),
            Exp::TypeUniv(e) => e.rename_in_ctx(ctx),
            Exp::Call(e) => e.rename_in_ctx(ctx),
            Exp::LocalMatch(e) => e.rename_in_ctx(ctx),
            Exp::DotCall(e) => e.rename_in_ctx(ctx),
        }
    }
}

impl Rename for TypeUniv {
    fn rename_in_ctx(&mut self, _ctx: &mut Ctx) {}
}

impl Rename for Call {
    fn rename_in_ctx(&mut self, ctx: &mut Ctx) {
        self.args.rename_in_ctx(ctx);
        self.inferred_type.rename_in_ctx(ctx);
    }
}

impl Rename for DotCall {
    fn rename_in_ctx(&mut self, ctx: &mut Ctx) {
        self.exp.rename_in_ctx(ctx);
        self.args.rename_in_ctx(ctx);
        self.inferred_type.rename_in_ctx(ctx);
    }
}

impl Rename for Variable {
    fn rename_in_ctx(&mut self, ctx: &mut Ctx) {
        let name = match ctx.binders.lookup(self.idx).name {
            VarBind::Var { id, .. } => id,
            VarBind::Wildcard { .. } => {
                // Currently, any wildcards are replaced by named variables when binding them to the context.
                // Therefore this case is unreachable.
                // In the future, we may want to allow wildcards to survive renaming.
                unreachable!()
            }
        };
        self.name = VarBound::from_string(&name);
        self.inferred_type.rename_in_ctx(ctx);
    }
}

impl Rename for LocalComatch {
    fn rename_in_ctx(&mut self, ctx: &mut Ctx) {
        self.ctx = None;
        self.inferred_type = None;
        self.cases.rename_in_ctx(ctx);
    }
}

impl Rename for Hole {
    fn rename_in_ctx(&mut self, ctx: &mut Ctx) {
        self.inferred_ctx = None;
        self.inferred_type.rename_in_ctx(ctx);
        self.args.rename_in_ctx(ctx);
    }
}

impl<T: Rename> Rename for Binder<T> {
    fn rename_in_ctx(&mut self, ctx: &mut Ctx) {
        // In expressions, `Binder` is currently only used to track the names of hole arguments.
        // These do not need to be renamed because they don't show up in the printed output.
        // They are solely used for better error messages.
        let Binder { name: _, content } = self;

        content.rename_in_ctx(ctx);
    }
}

impl Rename for LocalMatch {
    fn rename_in_ctx(&mut self, ctx: &mut Ctx) {
        self.ctx = None;
        self.inferred_type = None;
        self.on_exp.rename_in_ctx(ctx);
        self.motive.rename_in_ctx(ctx);
        self.ret_typ.rename_in_ctx(ctx);
        self.cases.rename_in_ctx(ctx);
    }
}

impl Rename for TypCtor {
    fn rename_in_ctx(&mut self, ctx: &mut Ctx) {
        self.args.rename_in_ctx(ctx);
    }
}

impl Rename for Anno {
    fn rename_in_ctx(&mut self, ctx: &mut Ctx) {
        self.exp.rename_in_ctx(ctx);
        self.typ.rename_in_ctx(ctx);
        self.normalized_type.rename_in_ctx(ctx);
    }
}

impl Rename for Args {
    fn rename_in_ctx(&mut self, ctx: &mut Ctx) {
        self.args.rename_in_ctx(ctx);
    }
}

impl Rename for Arg {
    fn rename_in_ctx(&mut self, ctx: &mut Ctx) {
        match self {
            Arg::UnnamedArg { arg, .. } => arg.rename_in_ctx(ctx),
            Arg::NamedArg { arg, .. } => arg.rename_in_ctx(ctx),
            Arg::InsertedImplicitArg { hole, .. } => hole.rename_in_ctx(ctx),
        }
    }
}

impl Rename for Case {
    fn rename_in_ctx(&mut self, ctx: &mut Ctx) {
        self.pattern.params.rename_in_ctx(ctx);

        ctx.bind_iter(self.pattern.params.params.iter(), |new_ctx| {
            self.body.rename_in_ctx(new_ctx);
        })
    }
}

impl Rename for Motive {
    fn rename_in_ctx(&mut self, ctx: &mut Ctx) {
        self.param.rename_in_ctx(ctx);
        ctx.bind_single(&self.param, |new_ctx| {
            self.ret_typ.rename_in_ctx(new_ctx);
        })
    }
}

impl Rename for Option<Span> {
    fn rename_in_ctx(&mut self, _ctx: &mut crate::Ctx) {}
}

impl<T: Rename> Rename for Option<T> {
    fn rename_in_ctx(&mut self, ctx: &mut Ctx) {
        match self {
            None => (),
            Some(x) => x.rename_in_ctx(ctx),
        }
    }
}

impl<T: Rename> Rename for Vec<T> {
    fn rename_in_ctx(&mut self, ctx: &mut Ctx) {
        self.iter_mut().for_each(|x| x.rename_in_ctx(ctx))
    }
}

impl Rename for () {
    fn rename_in_ctx(&mut self, _ctx: &mut Ctx) {}
}

impl Rename for Infix {
    fn rename_in_ctx(&mut self, _ctx: &mut Ctx) {}
}
