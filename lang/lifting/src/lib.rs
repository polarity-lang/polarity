use std::rc::Rc;

use renaming::Rename;
use syntax::common::*;
use syntax::ctx::values::TypeCtx;
use syntax::ctx::BindContext;
use syntax::ctx::LevelCtx;
use syntax::tst;
use syntax::ust;

mod fv;

use fv::*;

/// Lift local (co)matches for `name` in `prg` to top-level (co)definitions
pub fn lift(prg: tst::Prg, name: &str) -> LiftResult {
    let mut ctx = Ctx {
        name: name.to_owned(),
        new_decls: vec![],
        curr_decl: "".to_owned(),
        modified_decls: HashSet::default(),
        ctx: LevelCtx::default(),
    };

    let prg = prg.lift(&mut ctx).rename();
    let new_decls = HashSet::from_iter(ctx.new_decls.iter().map(|decl| decl.name().clone()));

    LiftResult { prg, new_decls, modified_decls: ctx.modified_decls }
}

/// Result of lifting
pub struct LiftResult {
    /// The resulting program
    pub prg: ust::Prg,
    /// List of new top-level definitions
    pub new_decls: HashSet<Ident>,
    /// List of top-level declarations that have been modified in the lifting process
    pub modified_decls: HashSet<Ident>,
}

#[derive(Debug)]
struct Ctx {
    /// The type name that should be lifted
    name: String,
    /// List of new top-level declarations that got created in the lifting process
    new_decls: Vec<ust::Decl>,
    /// Current declaration being visited for lifting
    curr_decl: Ident,
    /// List of declarations that got modified in the lifting process
    modified_decls: HashSet<Ident>,
    /// Tracks the current binders in scope
    ctx: LevelCtx,
}

impl BindContext for Ctx {
    type Ctx = LevelCtx;

    fn ctx_mut(&mut self) -> &mut Self::Ctx {
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

impl Lift for tst::Prg {
    type Target = ust::Prg;

    fn lift(&self, ctx: &mut Ctx) -> Self::Target {
        let tst::Prg { decls } = self;

        ust::Prg { decls: decls.lift(ctx) }
    }
}

impl Lift for tst::Decls {
    type Target = ust::Decls;

    fn lift(&self, ctx: &mut Ctx) -> Self::Target {
        let tst::Decls { map, lookup_table } = self;

        let mut map: HashMap<_, _> =
            map.iter().map(|(name, decl)| (name.clone(), decl.lift(ctx))).collect();
        let mut lookup_table = lookup_table.clone();

        // Add new top-level definitions to lookup tabble
        for decl in &ctx.new_decls {
            lookup_table.insert_def(ctx.name.clone(), decl.name().clone());
        }

        // Add new top-level definitions to program map
        let decls_iter = ctx.new_decls.iter().map(|decl| (decl.name().clone(), decl.clone()));
        map.extend(decls_iter);

        ust::Decls { map, lookup_table }
    }
}

impl Lift for tst::Decl {
    type Target = ust::Decl;

    fn lift(&self, ctx: &mut Ctx) -> Self::Target {
        ctx.set_curr_decl(self.name().clone());
        match self {
            tst::Decl::Data(data) => ust::Decl::Data(data.lift(ctx)),
            tst::Decl::Codata(cotata) => ust::Decl::Codata(cotata.lift(ctx)),
            tst::Decl::Ctor(ctor) => ust::Decl::Ctor(ctor.lift(ctx)),
            tst::Decl::Dtor(tdor) => ust::Decl::Dtor(tdor.lift(ctx)),
            tst::Decl::Def(def) => ust::Decl::Def(def.lift(ctx)),
            tst::Decl::Codef(codef) => ust::Decl::Codef(codef.lift(ctx)),
            tst::Decl::Let(tl_let) => ust::Decl::Let(tl_let.lift(ctx)),
        }
    }
}

impl Lift for tst::Data {
    type Target = ust::Data;

    fn lift(&self, ctx: &mut Ctx) -> Self::Target {
        let tst::Data { info, doc, name, attr, typ, ctors } = self;

        ust::Data {
            info: *info,
            doc: doc.clone(),
            name: name.clone(),
            attr: attr.clone(),
            typ: typ.lift(ctx),
            ctors: ctors.clone(),
        }
    }
}

impl Lift for tst::Codata {
    type Target = ust::Codata;

    fn lift(&self, ctx: &mut Ctx) -> Self::Target {
        let tst::Codata { info, doc, name, attr, typ, dtors } = self;

        ust::Codata {
            info: *info,
            doc: doc.clone(),
            name: name.clone(),
            attr: attr.clone(),
            typ: typ.lift(ctx),
            dtors: dtors.clone(),
        }
    }
}

impl Lift for tst::TypAbs {
    type Target = ust::TypAbs;

    fn lift(&self, ctx: &mut Ctx) -> Self::Target {
        let tst::TypAbs { params } = self;

        ust::TypAbs { params: params.lift_telescope(ctx, |_, params| params) }
    }
}

impl Lift for tst::Ctor {
    type Target = ust::Ctor;

    fn lift(&self, ctx: &mut Ctx) -> Self::Target {
        let tst::Ctor { info, doc, name, params, typ } = self;

        params.lift_telescope(ctx, |ctx, params| ust::Ctor {
            info: *info,
            doc: doc.clone(),
            name: name.clone(),
            params,
            typ: typ.lift(ctx),
        })
    }
}

impl Lift for tst::Dtor {
    type Target = ust::Dtor;

    fn lift(&self, ctx: &mut Ctx) -> Self::Target {
        let tst::Dtor { info, doc, name, params, self_param, ret_typ } = self;

        params.lift_telescope(ctx, |ctx, params| {
            let (self_param, ret_typ) = self_param.lift_telescope(ctx, |ctx, self_param| {
                let ret_typ = ret_typ.lift(ctx);
                (self_param, ret_typ)
            });
            ust::Dtor {
                info: *info,
                doc: doc.clone(),
                name: name.clone(),
                params,
                self_param,
                ret_typ,
            }
        })
    }
}

impl Lift for tst::Def {
    type Target = ust::Def;

    fn lift(&self, ctx: &mut Ctx) -> Self::Target {
        let tst::Def { info, doc, name, attr, params, self_param, ret_typ, body } = self;

        params.lift_telescope(ctx, |ctx, params| {
            let (self_param, ret_typ) = self_param.lift_telescope(ctx, |ctx, self_param| {
                let ret_typ = ret_typ.lift(ctx);
                (self_param, ret_typ)
            });

            ust::Def {
                info: *info,
                doc: doc.clone(),
                name: name.clone(),
                attr: attr.clone(),
                params,
                self_param,
                ret_typ,
                body: body.lift(ctx),
            }
        })
    }
}

impl Lift for tst::Codef {
    type Target = ust::Codef;

    fn lift(&self, ctx: &mut Ctx) -> Self::Target {
        let tst::Codef { info, doc, name, attr, params, typ, body } = self;

        params.lift_telescope(ctx, |ctx, params| ust::Codef {
            info: *info,
            doc: doc.clone(),
            name: name.clone(),
            attr: attr.clone(),
            params,
            typ: typ.lift(ctx),
            body: body.lift(ctx),
        })
    }
}

impl Lift for tst::Let {
    type Target = ust::Let;

    fn lift(&self, ctx: &mut Ctx) -> Self::Target {
        todo!()
    }
}

impl Lift for tst::Match {
    type Target = ust::Match;

    fn lift(&self, ctx: &mut Ctx) -> Self::Target {
        let tst::Match { info, cases, omit_absurd } = self;

        ust::Match { info: *info, cases: cases.lift(ctx), omit_absurd: *omit_absurd }
    }
}

impl Lift for tst::Case {
    type Target = ust::Case;

    fn lift(&self, ctx: &mut Ctx) -> Self::Target {
        let tst::Case { info, name, args, body } = self;

        args.lift_telescope(ctx, |ctx, args| ust::Case {
            info: *info,
            name: name.clone(),
            args,
            body: body.lift(ctx),
        })
    }
}

impl LiftTelescope for tst::SelfParam {
    type Target = ust::SelfParam;

    fn lift_telescope<T, F: FnOnce(&mut Ctx, Self::Target) -> T>(&self, ctx: &mut Ctx, f: F) -> T {
        let tst::SelfParam { info, name, typ } = self;

        ctx.bind_single((), |ctx| {
            let self_param = ust::SelfParam { info: *info, name: name.clone(), typ: typ.lift(ctx) };
            f(ctx, self_param)
        })
    }
}

impl Lift for tst::TypApp {
    type Target = ust::TypApp;

    fn lift(&self, ctx: &mut Ctx) -> Self::Target {
        let tst::TypApp { info, name, args } = self;

        ust::TypApp { info: info.forget(), name: name.clone(), args: args.lift(ctx) }
    }
}

impl Lift for tst::Exp {
    type Target = ust::Exp;

    fn lift(&self, ctx: &mut Ctx) -> Self::Target {
        match self {
            tst::Exp::Var { info, name, ctx: _, idx } => {
                ust::Exp::Var { info: info.forget(), name: name.clone(), ctx: (), idx: *idx }
            }
            tst::Exp::TypCtor { info, name, args } => {
                ust::Exp::TypCtor { info: info.forget(), name: name.clone(), args: args.lift(ctx) }
            }
            tst::Exp::Ctor { info, name, args } => {
                ust::Exp::Ctor { info: info.forget(), name: name.clone(), args: args.lift(ctx) }
            }
            tst::Exp::Dtor { info, exp, name, args } => ust::Exp::Dtor {
                info: info.forget(),
                exp: exp.lift(ctx),
                name: name.clone(),
                args: args.lift(ctx),
            },
            tst::Exp::Anno { info, exp, typ } => {
                ust::Exp::Anno { info: info.forget(), exp: exp.lift(ctx), typ: typ.lift(ctx) }
            }
            tst::Exp::Type { info } => ust::Exp::Type { info: info.forget() },
            tst::Exp::Hole { info } => ust::Exp::Hole { info: info.forget() },
            tst::Exp::Match { info, ctx: type_ctx, name, on_exp, motive, ret_typ, body } => {
                ctx.lift_match(info, type_ctx, name, on_exp, motive, ret_typ, body)
            }
            tst::Exp::Comatch { info, ctx: type_ctx, name, is_lambda_sugar, body } => {
                ctx.lift_comatch(info, type_ctx, name, *is_lambda_sugar, body)
            }
        }
    }
}

impl Lift for tst::Motive {
    type Target = ust::Motive;

    fn lift(&self, ctx: &mut Ctx) -> Self::Target {
        let tst::Motive { info, param, ret_typ } = self;

        let param = param.lift(ctx);

        ctx.bind_single((), |ctx| ust::Motive { info: *info, param, ret_typ: ret_typ.lift(ctx) })
    }
}

impl LiftTelescope for tst::Telescope {
    type Target = ust::Telescope;

    fn lift_telescope<T, F: FnOnce(&mut Ctx, Self::Target) -> T>(&self, ctx: &mut Ctx, f: F) -> T {
        let tst::Telescope { params } = self;

        ctx.bind_fold(
            params.iter(),
            vec![],
            |ctx, mut acc, param| {
                acc.push(param.lift(ctx));
                acc
            },
            |ctx, params| f(ctx, ust::Telescope { params }),
        )
    }
}

impl LiftTelescope for tst::TelescopeInst {
    type Target = ust::TelescopeInst;

    fn lift_telescope<T, F: FnOnce(&mut Ctx, Self::Target) -> T>(&self, ctx: &mut Ctx, f: F) -> T {
        let tst::TelescopeInst { params } = self;

        ctx.bind_fold(
            params.iter(),
            vec![],
            |ctx, mut acc, param| {
                acc.push(param.lift(ctx));
                acc
            },
            |ctx, params| f(ctx, ust::TelescopeInst { params }),
        )
    }
}

impl Lift for tst::Args {
    type Target = ust::Args;

    fn lift(&self, ctx: &mut Ctx) -> Self::Target {
        let tst::Args { args } = self;

        ust::Args { args: args.lift(ctx) }
    }
}

impl Lift for tst::Param {
    type Target = ust::Param;

    fn lift(&self, ctx: &mut Ctx) -> Self::Target {
        let tst::Param { name, typ } = self;

        ust::Param { name: name.clone(), typ: typ.lift(ctx) }
    }
}

impl Lift for tst::ParamInst {
    type Target = ust::ParamInst;

    fn lift(&self, _ctx: &mut Ctx) -> Self::Target {
        let tst::ParamInst { info, name, typ: _ } = self;

        ust::ParamInst { info: info.forget(), name: name.clone(), typ: () }
    }
}

impl<T: Lift> Lift for Rc<T> {
    type Target = Rc<T::Target>;

    fn lift(&self, ctx: &mut Ctx) -> Self::Target {
        Rc::new(T::lift(self, ctx))
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
        info: &tst::TypeAppInfo,
        type_ctx: &TypeCtx,
        name: &Label,
        on_exp: &Rc<tst::Exp>,
        motive: &Option<tst::Motive>,
        ret_typ: &tst::Typ,
        body: &tst::Match,
    ) -> ust::Exp {
        // Only lift local matches for the specified type
        if info.typ.name != self.name {
            return ust::Exp::Match {
                info: info.forget(),
                ctx: (),
                name: name.clone(),
                on_exp: on_exp.lift(self),
                motive: motive.lift(self),
                ret_typ: (),
                body: body.lift(self),
            };
        }

        self.mark_modified();

        // Collect the free variables in the match,
        // the type of the scrutinee as well as the return type of the match
        // Build a telescope of the types of the lifted variables
        let ret_fvs = motive
            .as_ref()
            .map(|m| m.forget().free_vars(type_ctx))
            .unwrap_or_else(|| ret_typ.as_exp().forget().free_vars(type_ctx));

        let body = body.lift(self);
        let self_typ = info.typ.lift(self);

        let FreeVarsResult { telescope, subst, args } = body
            .free_vars(type_ctx)
            .union(self_typ.free_vars(type_ctx))
            .union(ret_fvs)
            .telescope(&self.ctx);

        // Substitute the new parameters for the free variables
        let body = body.subst(&mut self.ctx, &subst.in_body());
        let self_typ = self_typ.subst(&mut self.ctx, &subst.in_body());
        let def_ret_typ = match &motive {
            Some(m) => m.lift(self).subst(&mut self.ctx, &subst.in_body()).ret_typ,
            None => {
                ret_typ.as_exp().lift(self).subst(&mut self.ctx, &subst.in_body()).shift((1, 0))
            }
        };

        // Build the new top-level definition
        let name = self.unique_def_name(name, &info.typ.name);

        let def = ust::Def {
            info: None,
            doc: None,
            name: name.clone(),
            attr: ust::Attribute::default(),
            params: telescope,
            self_param: ust::SelfParam {
                info: None,
                name: motive.as_ref().map(|m| m.param.name.clone()),
                typ: self_typ,
            },
            ret_typ: def_ret_typ,
            body,
        };

        self.new_decls.push(ust::Decl::Def(def));

        // Replace the match by a destructor call of the new top-level definition
        ust::Exp::Dtor { info: None, exp: on_exp.lift(self), name, args }
    }

    fn lift_comatch(
        &mut self,
        info: &tst::TypeAppInfo,
        type_ctx: &TypeCtx,
        name: &Label,
        is_lambda_sugar: bool,
        body: &tst::Match,
    ) -> ust::Exp {
        // Only lift local matches for the specified type
        if info.typ.name != self.name {
            return ust::Exp::Comatch {
                info: info.forget(),
                ctx: (),
                name: name.clone(),
                is_lambda_sugar,
                body: body.lift(self),
            };
        }

        self.mark_modified();

        let body = body.lift(self);
        let typ = info.typ.lift(self);

        // Collect the free variables in the comatch and the return type
        // Build a telescope of the types of the lifted variables
        let FreeVarsResult { telescope, subst, args } =
            body.free_vars(type_ctx).union(typ.free_vars(type_ctx)).telescope(&self.ctx);

        // Substitute the new parameters for the free variables
        let body = body.subst(&mut self.ctx, &subst.in_body());
        let typ = typ.subst(&mut self.ctx, &subst.in_body());

        // Build the new top-level definition
        let name = self.unique_codef_name(name, &info.typ.name);

        let codef = ust::Codef {
            info: None,
            doc: None,
            name: name.clone(),
            attr: ust::Attribute::default(),
            params: telescope,
            typ,
            body,
        };

        self.new_decls.push(ust::Decl::Codef(codef));

        // Replace the comatch by a call of the new top-level definition
        ust::Exp::Ctor { info: None, name, args }
    }

    /// Set the current declaration
    fn set_curr_decl(&mut self, name: Ident) {
        self.curr_decl = name;
    }

    /// Mark the current declaration as modified
    fn mark_modified(&mut self) {
        self.modified_decls.insert(self.curr_decl.clone());
    }

    /// Generate a definition name based on the label and type information
    fn unique_def_name(&self, label: &Label, type_name: &str) -> Ident {
        label.user_name.clone().unwrap_or_else(|| {
            let lowered = type_name.to_lowercase();
            let id = label.id;
            format!("d_{lowered}{id}")
        })
    }

    /// Generate a codefinition name based on the label and type information
    fn unique_codef_name(&self, label: &Label, type_name: &str) -> Ident {
        label.user_name.clone().unwrap_or_else(|| {
            let id = label.id;
            format!("Mk{type_name}{id}")
        })
    }
}
