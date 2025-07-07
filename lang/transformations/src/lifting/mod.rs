use std::sync::Arc;

use ast::rename::Rename;
use url::Url;

use ast::ctx::BindContext;
use ast::ctx::LevelCtx;
use ast::ctx::values::TypeCtx;
use ast::*;
use ctx::values::Binder;
use miette_util::codespan::Span;

mod fv;
mod signature;

use fv::*;
use signature::*;

/// Lift local (co)matches for `name` in `module` to top-level (co)definitions
pub fn lift(module: Arc<Module>, name: &str) -> LiftResult {
    let mut ctx = Ctx {
        name: name.to_owned(),
        new_decls: vec![],
        curr_decl: IdBind::from_string(""),
        modified_decls: HashSet::default(),
        ctx: LevelCtx::empty(),
        uri: module.uri.clone(),
    };

    let mut module = module.lift(&mut ctx);

    let mut new_decl_names: HashSet<IdBind> = Default::default();
    for decl in ctx.new_decls.iter() {
        if let Some(id) = decl.ident() {
            new_decl_names.insert(id.clone());
        }
    }
    module.decls.extend(ctx.new_decls);
    module.rename();

    LiftResult { module, new_decls: new_decl_names, modified_decls: ctx.modified_decls }
}

/// Result of lifting
pub struct LiftResult {
    /// The resulting program
    pub module: Module,
    /// List of new top-level definitions
    pub new_decls: HashSet<IdBind>,
    /// List of top-level declarations that have been modified in the lifting process
    pub modified_decls: HashSet<IdBind>,
}

#[derive(Debug)]
struct Ctx {
    /// The type name that should be lifted
    name: String,
    /// List of new top-level declarations that got created in the lifting process
    new_decls: Vec<Decl>,
    /// Current declaration being visited for lifting
    curr_decl: IdBind,
    /// List of declarations that got modified in the lifting process
    modified_decls: HashSet<IdBind>,
    /// Tracks the current binders in scope
    ctx: LevelCtx,
    /// URI of the current module
    uri: Url,
}

impl BindContext for Ctx {
    type Content = ();

    fn ctx_mut(&mut self) -> &mut LevelCtx {
        &mut self.ctx
    }
}

trait Lift {
    type Target;

    fn lift(&self, ctx: &mut Ctx) -> Self::Target;
}

trait LiftTelescope {
    type Target;

    fn lift_telescope<T, F: FnOnce(&mut Ctx, Self::Target) -> T>(&self, ctx: &mut Ctx, f: F) -> T;
}

impl Lift for Module {
    type Target = Module;

    fn lift(&self, ctx: &mut Ctx) -> Self::Target {
        let Module { uri, use_decls, decls, meta_vars } = self;

        let decls = decls.iter().map(|decl| decl.lift(ctx)).collect();

        Module {
            uri: uri.clone(),
            use_decls: use_decls.clone(),
            decls,
            meta_vars: meta_vars.clone(),
        }
    }
}

impl Lift for Decl {
    type Target = Decl;

    fn lift(&self, ctx: &mut Ctx) -> Self::Target {
        match self {
            Decl::Data(data) => Decl::Data(data.lift(ctx)),
            Decl::Codata(cotata) => Decl::Codata(cotata.lift(ctx)),
            Decl::Def(def) => Decl::Def(def.lift(ctx)),
            Decl::Codef(codef) => Decl::Codef(codef.lift(ctx)),
            Decl::Let(tl_let) => Decl::Let(tl_let.lift(ctx)),
            Decl::Infix(infix) => Decl::Infix(infix.lift(ctx)),
        }
    }
}

impl Lift for Data {
    type Target = Data;

    fn lift(&self, ctx: &mut Ctx) -> Self::Target {
        let Data { span, doc, name, attr, typ, ctors } = self;
        ctx.set_curr_decl(name.clone());

        let ctors = ctors.iter().map(|ctor| ctor.lift(ctx)).collect();

        Data {
            span: *span,
            doc: doc.clone(),
            name: name.clone(),
            attr: attr.clone(),
            typ: Box::new(typ.lift_telescope(ctx, |_, params| params)),
            ctors,
        }
    }
}

impl Lift for Codata {
    type Target = Codata;

    fn lift(&self, ctx: &mut Ctx) -> Self::Target {
        let Codata { span, doc, name, attr, typ, dtors } = self;
        ctx.set_curr_decl(name.clone());

        let dtors = dtors.iter().map(|dtor| dtor.lift(ctx)).collect();

        Codata {
            span: *span,
            doc: doc.clone(),
            name: name.clone(),
            attr: attr.clone(),
            typ: Box::new(typ.lift_telescope(ctx, |_, params| params)),
            dtors,
        }
    }
}

impl Lift for Ctor {
    type Target = Ctor;

    fn lift(&self, ctx: &mut Ctx) -> Self::Target {
        let Ctor { span, doc, name, params, typ } = self;

        params.lift_telescope(ctx, |ctx, params| Ctor {
            span: *span,
            doc: doc.clone(),
            name: name.clone(),
            params,
            typ: typ.lift(ctx),
        })
    }
}

impl Lift for Dtor {
    type Target = Dtor;

    fn lift(&self, ctx: &mut Ctx) -> Self::Target {
        let Dtor { span, doc, name, params, self_param, ret_typ } = self;

        params.lift_telescope(ctx, |ctx, params| {
            let (self_param, ret_typ) = self_param.lift_telescope(ctx, |ctx, self_param| {
                let ret_typ = ret_typ.lift(ctx);
                (self_param, ret_typ)
            });
            Dtor { span: *span, doc: doc.clone(), name: name.clone(), params, self_param, ret_typ }
        })
    }
}

impl Lift for Def {
    type Target = Def;

    fn lift(&self, ctx: &mut Ctx) -> Self::Target {
        let Def { span, doc, name, attr, params, self_param, ret_typ, cases } = self;
        ctx.set_curr_decl(name.clone());

        params.lift_telescope(ctx, |ctx, params| {
            let (self_param, ret_typ) = self_param.lift_telescope(ctx, |ctx, self_param| {
                let ret_typ = ret_typ.lift(ctx);
                (self_param, ret_typ)
            });

            Def {
                span: *span,
                doc: doc.clone(),
                name: name.clone(),
                attr: attr.clone(),
                params,
                self_param,
                ret_typ,
                cases: cases.lift(ctx),
            }
        })
    }
}

impl Lift for Codef {
    type Target = Codef;

    fn lift(&self, ctx: &mut Ctx) -> Self::Target {
        let Codef { span, doc, name, attr, params, typ, cases } = self;
        ctx.set_curr_decl(name.clone());

        params.lift_telescope(ctx, |ctx, params| Codef {
            span: *span,
            doc: doc.clone(),
            name: name.clone(),
            attr: attr.clone(),
            params,
            typ: typ.lift(ctx),
            cases: cases.lift(ctx),
        })
    }
}

impl Lift for Let {
    type Target = Let;

    fn lift(&self, ctx: &mut Ctx) -> Self::Target {
        let Let { span, doc, name, attr, params, typ, body } = self;
        ctx.set_curr_decl(name.clone());

        params.lift_telescope(ctx, |ctx, params| Let {
            span: *span,
            doc: doc.clone(),
            name: name.clone(),
            attr: attr.clone(),
            params,
            typ: typ.lift(ctx),
            body: body.lift(ctx),
        })
    }
}

impl Lift for Case {
    type Target = Case;

    fn lift(&self, ctx: &mut Ctx) -> Self::Target {
        let Case { span, pattern, body } = self;

        pattern.params.lift_telescope(ctx, |ctx, params| Case {
            span: *span,
            pattern: Pattern {
                span: None,
                is_copattern: pattern.is_copattern,
                name: pattern.name.clone(),
                params,
            },
            body: body.lift(ctx),
        })
    }
}

impl LiftTelescope for SelfParam {
    type Target = SelfParam;

    fn lift_telescope<T, F: FnOnce(&mut Ctx, Self::Target) -> T>(&self, ctx: &mut Ctx, f: F) -> T {
        let SelfParam { info, name, typ } = self;

        ctx.bind_single(name.clone(), |ctx| {
            let self_param = SelfParam { info: *info, name: name.clone(), typ: typ.lift(ctx) };
            f(ctx, self_param)
        })
    }
}

impl Lift for Exp {
    type Target = Exp;

    fn lift(&self, ctx: &mut Ctx) -> Self::Target {
        match self {
            Exp::Variable(e) => e.lift(ctx),
            Exp::TypCtor(e) => Exp::TypCtor(e.lift(ctx)),
            Exp::Call(e) => e.lift(ctx),
            Exp::DotCall(e) => e.lift(ctx),
            Exp::Anno(e) => e.lift(ctx),
            Exp::TypeUniv(e) => e.lift(ctx),
            Exp::Hole(e) => e.lift(ctx).into(),
            Exp::LocalMatch(e) => e.lift(ctx),
            Exp::LocalComatch(e) => e.lift(ctx),
            Exp::LocalLet(e) => e.lift(ctx),
        }
    }
}

impl Lift for Variable {
    type Target = Exp;

    fn lift(&self, _ctx: &mut Ctx) -> Self::Target {
        let Variable { span, idx, name, .. } = self;
        Exp::Variable(Variable { span: *span, idx: *idx, name: name.clone(), inferred_type: None })
    }
}

impl Lift for TypCtor {
    type Target = TypCtor;

    fn lift(&self, ctx: &mut Ctx) -> Self::Target {
        let TypCtor { span, name, args, is_bin_op } = self;
        TypCtor {
            span: *span,
            name: name.clone(),
            args: args.lift(ctx),
            is_bin_op: is_bin_op.clone(),
        }
    }
}

impl Lift for Call {
    type Target = Exp;

    fn lift(&self, ctx: &mut Ctx) -> Self::Target {
        let Call { span, name, args, kind, .. } = self;
        Exp::Call(Call {
            span: *span,
            kind: *kind,
            name: name.clone(),
            args: args.lift(ctx),
            inferred_type: None,
        })
    }
}

impl Lift for DotCall {
    type Target = Exp;

    fn lift(&self, ctx: &mut Ctx) -> Self::Target {
        let DotCall { span, kind, exp, name, args, .. } = self;
        Exp::DotCall(DotCall {
            span: *span,
            kind: *kind,
            exp: exp.lift(ctx),
            name: name.clone(),
            args: args.lift(ctx),
            inferred_type: None,
        })
    }
}

impl Lift for Anno {
    type Target = Exp;

    fn lift(&self, ctx: &mut Ctx) -> Self::Target {
        let Anno { span, exp, typ, .. } = self;
        Exp::Anno(Anno {
            span: *span,
            exp: exp.lift(ctx),
            typ: typ.lift(ctx),
            normalized_type: None,
        })
    }
}

impl Lift for TypeUniv {
    type Target = Exp;

    fn lift(&self, _ctx: &mut Ctx) -> Self::Target {
        let TypeUniv { span } = self;
        Exp::TypeUniv(TypeUniv { span: *span })
    }
}

impl Lift for Hole {
    type Target = Hole;

    fn lift(&self, _ctx: &mut Ctx) -> Self::Target {
        let Hole { span, kind, metavar, args, .. } = self;
        Hole {
            span: *span,
            kind: *kind,
            metavar: *metavar,
            inferred_type: None,
            inferred_ctx: None,
            args: args.clone(),
            solution: None,
        }
    }
}

impl Lift for LocalMatch {
    type Target = Exp;

    fn lift(&self, ctx: &mut Ctx) -> Self::Target {
        let LocalMatch { span, ctx: type_ctx, name, on_exp, motive, ret_typ, cases, inferred_type } =
            self;
        ctx.lift_match(
            span,
            &inferred_type.clone().unwrap(),
            &type_ctx.clone().unwrap(),
            name,
            on_exp,
            motive,
            ret_typ,
            cases,
        )
    }
}

impl Lift for LocalComatch {
    type Target = Exp;

    fn lift(&self, ctx: &mut Ctx) -> Self::Target {
        let LocalComatch { span, ctx: type_ctx, name, is_lambda_sugar, cases, inferred_type } =
            self;
        ctx.lift_comatch(
            span,
            &inferred_type.clone().unwrap(),
            &type_ctx.clone().unwrap(),
            name,
            *is_lambda_sugar,
            cases,
        )
    }
}

impl Lift for LocalLet {
    type Target = Exp;

    fn lift(&self, ctx: &mut Ctx) -> Self::Target {
        let LocalLet { span, name, typ, bound, body, inferred_type: _ } = self;

        Exp::LocalLet(LocalLet {
            span: *span,
            name: name.clone(),
            typ: typ.lift(ctx),
            bound: bound.lift(ctx),
            body: ctx.bind_single(name.clone(), |ctx| body.lift(ctx)),
            inferred_type: None,
        })
    }
}

impl Lift for Motive {
    type Target = Motive;

    fn lift(&self, ctx: &mut Ctx) -> Self::Target {
        let Motive { span, param, ret_typ } = self;

        let param = param.lift(ctx);

        let binder = Binder { name: param.name.clone(), content: () };
        ctx.bind_single(&binder, |ctx| Motive { span: *span, param, ret_typ: ret_typ.lift(ctx) })
    }
}

impl LiftTelescope for Telescope {
    type Target = Telescope;

    fn lift_telescope<T, F: FnOnce(&mut Ctx, Self::Target) -> T>(&self, ctx: &mut Ctx, f: F) -> T {
        let Telescope { params } = self;

        ctx.bind_fold(
            params.iter(),
            vec![],
            |ctx, acc, param| {
                acc.push(param.lift(ctx));
                Binder { name: param.name.clone(), content: () }
            },
            |ctx, params| f(ctx, Telescope { params }),
        )
    }
}

impl LiftTelescope for TelescopeInst {
    type Target = TelescopeInst;

    fn lift_telescope<T, F: FnOnce(&mut Ctx, Self::Target) -> T>(&self, ctx: &mut Ctx, f: F) -> T {
        let TelescopeInst { params } = self;

        ctx.bind_fold(
            params.iter(),
            vec![],
            |ctx, acc, param| {
                acc.push(param.lift(ctx));
                Binder { name: param.name.clone(), content: () }
            },
            |ctx, params| f(ctx, TelescopeInst { params }),
        )
    }
}

impl Lift for Args {
    type Target = Args;

    fn lift(&self, ctx: &mut Ctx) -> Self::Target {
        let Args { args } = self;

        Args { args: args.lift(ctx) }
    }
}

impl Lift for Arg {
    type Target = Arg;

    fn lift(&self, ctx: &mut Ctx) -> Self::Target {
        match self {
            Arg::UnnamedArg { arg, erased } => {
                Arg::UnnamedArg { arg: arg.lift(ctx), erased: *erased }
            }
            Arg::NamedArg { name, arg, erased } => {
                Arg::NamedArg { name: name.clone(), arg: arg.lift(ctx), erased: *erased }
            }
            Arg::InsertedImplicitArg { hole, erased } => {
                Arg::InsertedImplicitArg { hole: hole.lift(ctx), erased: *erased }
            }
        }
    }
}

impl Lift for Param {
    type Target = Param;

    fn lift(&self, ctx: &mut Ctx) -> Self::Target {
        let Param { implicit, name, typ, erased } = self;

        Param { implicit: *implicit, name: name.clone(), typ: typ.lift(ctx), erased: *erased }
    }
}

impl Lift for ParamInst {
    type Target = ParamInst;

    fn lift(&self, _ctx: &mut Ctx) -> Self::Target {
        let ParamInst { span, name, typ: _, erased, .. } = self;

        ParamInst { span: *span, name: name.clone(), typ: None, erased: *erased }
    }
}

impl<T: Lift> Lift for Box<T> {
    type Target = Box<T::Target>;

    fn lift(&self, ctx: &mut Ctx) -> Self::Target {
        Box::new(T::lift(self, ctx))
    }
}

impl<T: Lift> Lift for Option<T> {
    type Target = Option<T::Target>;

    fn lift(&self, ctx: &mut Ctx) -> Self::Target {
        self.as_ref().map(|x| x.lift(ctx))
    }
}

impl<T: Lift> Lift for Vec<T> {
    type Target = Vec<T::Target>;

    fn lift(&self, ctx: &mut Ctx) -> Self::Target {
        self.iter().map(|x| x.lift(ctx)).collect()
    }
}

impl Ctx {
    #[allow(clippy::too_many_arguments)]
    fn lift_match(
        &mut self,
        span: &Option<Span>,
        inferred_type: &TypCtor,
        type_ctx: &TypeCtx,
        name: &Label,
        on_exp: &Exp,
        motive: &Option<Motive>,
        ret_typ: &Option<Box<Exp>>,
        cases: &Vec<Case>,
    ) -> Exp {
        // Only lift local matches for the specified type
        if inferred_type.name.id != self.name {
            return Exp::LocalMatch(LocalMatch {
                span: *span,
                inferred_type: None,
                ctx: None,
                name: name.clone(),
                on_exp: Box::new(on_exp.lift(self)),
                motive: motive.lift(self),
                ret_typ: None,
                cases: cases.lift(self),
            });
        }

        self.mark_modified();

        // Collect the free variables in the match,
        // the type of the scrutinee as well as the return type of the match
        // Build a telescope of the types of the lifted variables
        let ret_fvs = motive
            .as_ref()
            .map(|m| free_vars_closure(m, type_ctx))
            .unwrap_or_else(|| free_vars_closure(ret_typ, type_ctx));

        let cases = cases.lift(self);
        let self_typ = inferred_type.lift(self);

        let mut fvs = free_vars_closure(&cases, type_ctx);
        fvs.extend(free_vars_closure(&self_typ, type_ctx));
        fvs.extend(ret_fvs);

        let LiftedSignature { telescope, subst, args } = lifted_signature(fvs, &self.ctx);

        // Substitute the new parameters for the free variables
        // Unwrap is safe here because we are unwrapping an infallible result
        let cases = cases.subst(&mut self.ctx, &subst).unwrap();
        let self_typ = self_typ.subst(&mut self.ctx, &subst).unwrap();
        let def_ret_typ = match &motive {
            Some(m) => m.lift(self).subst(&mut self.ctx, &subst).unwrap().ret_typ,
            None => shift_and_clone(
                &ret_typ.clone().unwrap().lift(self).subst(&mut self.ctx, &subst).unwrap(),
                (1, 0),
            ),
        };

        // Build the new top-level definition
        let name = self.unique_def_name(name, &inferred_type.name.id);

        let def = Def {
            span: None,
            doc: None,
            name: name.clone(),
            attr: Attributes::default(),
            params: telescope,
            self_param: SelfParam {
                info: None,
                name: motive
                    .as_ref()
                    .map(|m| m.param.name.clone())
                    .unwrap_or(VarBind::Wildcard { span: None }),
                typ: self_typ,
            },
            ret_typ: def_ret_typ,
            cases,
        };

        self.new_decls.push(Decl::Def(def));

        // Replace the match by a dotcall of the new top-level definition
        Exp::DotCall(DotCall {
            span: None,
            kind: DotCallKind::Definition,
            exp: Box::new(on_exp.lift(self)),
            name: IdBound { span: None, id: name.id.clone(), uri: self.uri.clone() },
            args,
            inferred_type: None,
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn lift_comatch(
        &mut self,
        span: &Option<Span>,
        inferred_type: &TypCtor,
        type_ctx: &TypeCtx,
        name: &Label,
        is_lambda_sugar: bool,
        cases: &Vec<Case>,
    ) -> Exp {
        // Only lift local matches for the specified type
        if inferred_type.name.id != self.name {
            return Exp::LocalComatch(LocalComatch {
                span: *span,
                ctx: None,
                name: name.clone(),
                is_lambda_sugar,
                cases: cases.lift(self),
                inferred_type: None,
            });
        }

        self.mark_modified();

        let cases = cases.lift(self);
        let typ = inferred_type.lift(self);

        // Collect the free variables in the comatch and the return type
        let mut fvs = free_vars_closure(&cases, type_ctx);
        fvs.extend(free_vars_closure(&typ, type_ctx));

        // Build a telescope of the types of the lifted variables
        let LiftedSignature { telescope, subst, args } = lifted_signature(fvs, &self.ctx);

        // Substitute the new parameters for the free variables
        // Unwrap is safe here because we are unwrapping an infallible result
        let cases = cases.subst(&mut self.ctx, &subst).unwrap();
        let typ = typ.subst(&mut self.ctx, &subst).unwrap();

        // Build the new top-level definition
        let name = self.unique_codef_name(name, &inferred_type.name.id);

        let codef = Codef {
            span: None,
            doc: None,
            name: name.clone(),
            attr: Attributes::default(),
            params: telescope,
            typ,
            cases,
        };

        self.new_decls.push(Decl::Codef(codef));

        // Replace the comatch by a call of the new top-level codefinition
        Exp::Call(Call {
            span: None,
            kind: CallKind::Codefinition,
            name: IdBound { span: None, id: name.id.clone(), uri: self.uri.clone() },
            args,
            inferred_type: None,
        })
    }

    /// Set the current declaration
    fn set_curr_decl(&mut self, name: IdBind) {
        self.curr_decl = name;
    }

    /// Mark the current declaration as modified
    fn mark_modified(&mut self) {
        self.modified_decls.insert(self.curr_decl.clone());
    }

    /// Generate a definition name based on the label and type information
    fn unique_def_name(&self, label: &Label, type_name: &str) -> IdBind {
        label.user_name.clone().unwrap_or_else(|| {
            let lowered = type_name.to_lowercase();
            let id = label.id;
            IdBind::from_string(&format!("d_{lowered}{id}"))
        })
    }

    /// Generate a codefinition name based on the label and type information
    fn unique_codef_name(&self, label: &Label, type_name: &str) -> IdBind {
        label.user_name.clone().unwrap_or_else(|| {
            let id = label.id;
            IdBind::from_string(&format!("Mk{type_name}{id}"))
        })
    }
}

impl Lift for Infix {
    type Target = Infix;
    fn lift(&self, _ctx: &mut Ctx) -> Self::Target {
        self.clone()
    }
}
