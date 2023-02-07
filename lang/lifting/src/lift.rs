use std::rc::Rc;

use data::HashSet;
use renaming::Rename;
use syntax::ast::fv::FreeVarsExt;
use syntax::ast::fv::FreeVarsResult;
use syntax::ast::*;
use syntax::common::*;
use syntax::ctx::*;
use syntax::ust;
use syntax::wst;
use syntax::wst::WST;

#[derive(Debug)]
pub struct Lift {
    /// The type name that should be lifted
    name: String,
    /// List of new top-level declarations that got created in the lifting process
    new_decls: Vec<wst::Decl>,
    /// Typing context
    ctx: TypeCtx<WST>,
    /// Current declaration being visited for lifting
    curr_decl: Ident,
    /// List of declarations that got modified in the lifting process
    modified_decls: HashSet<Ident>,
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

impl Lift {
    pub fn new(name: String) -> Self {
        Self {
            name,
            new_decls: vec![],
            ctx: TypeCtx::empty(),
            curr_decl: Default::default(),
            modified_decls: Default::default(),
        }
    }

    pub fn run(mut self, prg: wst::Prg) -> LiftResult {
        // Perform the lifting process
        let mut prg_out = prg.map(&mut self);

        let new_decls = self.new_decls.iter().map(|decl| decl.name().clone()).collect();

        // Update program accordingly
        for decl in &self.new_decls {
            prg_out.decls.source.insert_def(self.name.clone(), decl.name().clone());
        }

        let decls_iter = self.new_decls.into_iter().map(|decl| (decl.name().clone(), decl));
        prg_out.decls.map.extend(decls_iter);

        // Rename program to ensure proper variable naming for lifted definitions
        // FIXME: While renaming takes care to rename in info annotations, substitution does not.
        // Hence, we first forget the type annotations before renaming
        let prg_out = prg_out.forget().rename();

        LiftResult { prg: prg_out, new_decls, modified_decls: self.modified_decls }
    }

    /// Set the current declaration
    fn set_curr_decl(&mut self, name: Ident) {
        self.curr_decl = name;
    }

    /// Mark the current declaration as modified
    fn mark_modified(&mut self) {
        self.modified_decls.insert(self.curr_decl.clone());
    }
}

impl Mapper<WST> for Lift {
    fn enter_decl(&mut self, decl: &wst::Decl) {
        self.set_curr_decl(decl.name().clone());
    }

    fn map_exp_match(
        &mut self,
        info: wst::TypeAppInfo,
        name: Ident,
        on_exp: Rc<wst::Exp>,
        motive: Option<wst::Motive>,
        ret_typ: wst::Typ,
        body: wst::Match,
    ) -> wst::Exp {
        // Only lift local matches for the specified type
        if info.typ.name != self.name {
            return id().map_exp_match(info, name, on_exp, motive, ret_typ, body);
        }

        self.mark_modified();

        // Collect the free variables in the match,
        // the type of the scrutinee as well as the return type of the match
        // Build a telescope of the types of the lifted variables
        let ret_fvs = motive
            .as_ref()
            .map(|m| m.free_vars(&mut self.ctx))
            .unwrap_or_else(|| ret_typ.as_exp().free_vars(&mut self.ctx));

        let FreeVarsResult { telescope, subst, args } = body
            .free_vars(&mut self.ctx)
            .union(info.typ.free_vars(&mut self.ctx))
            .union(ret_fvs)
            .telescope();

        // Substitute the new parameters for the free variables
        let body = body.subst(&mut self.ctx.levels(), &subst);
        let self_typ = info.typ.subst(&mut self.ctx.levels(), &subst);
        let def_ret_typ = match &motive {
            Some(m) => m.subst(&mut self.ctx.levels(), &subst).ret_typ,
            None => ret_typ.as_exp().subst(&mut self.ctx.levels(), &subst).shift((1, 0)),
        };

        // Build the new top-level definition
        let def = wst::Def {
            info: wst::Info::empty(),
            name: name.clone(),
            ignored: false,
            params: telescope,
            self_param: wst::SelfParam {
                info: wst::Info::empty(),
                name: motive.map(|m| m.param.name),
                typ: self_typ,
            },
            ret_typ: def_ret_typ,
            body,
        };

        self.new_decls.push(Decl::Def(def));

        // Replace the match by a destructor call of the new top-level definition
        wst::Exp::Dtor {
            info: wst::TypeInfo { typ: ret_typ.as_exp().forget(), span: Default::default() },
            exp: on_exp,
            name,
            args,
        }
    }

    fn map_exp_comatch(
        &mut self,
        info: wst::TypeAppInfo,
        name: Ident,
        body: wst::Comatch,
    ) -> wst::Exp {
        // Only lift local matches for the specified type
        if info.typ.name != self.name {
            return id().map_exp_comatch(info, name, body);
        }

        self.mark_modified();

        // Collect the free variables in the comatch and the return type
        // Build a telescope of the types of the lifted variables
        let FreeVarsResult { telescope, subst, args } =
            body.free_vars(&mut self.ctx).union(info.typ.free_vars(&mut self.ctx)).telescope();

        // Substitute the new parameters for the free variables
        let body = body.subst(&mut self.ctx.levels(), &subst);
        let typ = info.typ.subst(&mut self.ctx.levels(), &subst);

        // Build the new top-level definition
        let codef = wst::Codef {
            info: wst::Info::empty(),
            name: name.clone(),
            ignored: false,
            params: telescope,
            typ,
            body,
        };

        self.new_decls.push(Decl::Codef(codef));

        // Replace the comatch by a call of the new top-level definition
        wst::Exp::Ctor {
            info: wst::TypeInfo {
                typ: Rc::new(info.typ.to_exp().forget()),
                span: Default::default(),
            },
            name,
            args,
        }
    }

    fn map_telescope<X, I, F1, F2>(&mut self, params: I, f_acc: F1, f_inner: F2) -> X
    where
        I: IntoIterator<Item = wst::Param>,
        F1: Fn(&mut Self, wst::Param) -> wst::Param,
        F2: FnOnce(&mut Self, wst::Telescope) -> X,
    {
        self.ctx_map_telescope(params, f_acc, f_inner)
    }

    fn map_telescope_inst<X, I, F1, F2>(&mut self, params: I, f_acc: F1, f_inner: F2) -> X
    where
        I: IntoIterator<Item = wst::ParamInst>,
        F1: Fn(&mut Self, wst::ParamInst) -> wst::ParamInst,
        F2: FnOnce(&mut Self, wst::TelescopeInst) -> X,
    {
        self.ctx_map_telescope_inst(params, f_acc, f_inner)
    }

    fn map_motive_param<X, F>(&mut self, param: ParamInst<WST>, f_inner: F) -> X
    where
        F: FnOnce(&mut Self, ParamInst<WST>) -> X,
    {
        self.ctx_map_motive_param(param, f_inner)
    }

    fn map_self_param<X, F>(
        &mut self,
        info: <WST as Phase>::Info,
        name: Option<Ident>,
        typ: TypApp<WST>,
        f_inner: F,
    ) -> X
    where
        F: FnOnce(&mut Self, SelfParam<WST>) -> X,
    {
        self.ctx_map_self_param(info, name, typ, f_inner)
    }
}

impl HasContext for Lift {
    type Ctx = TypeCtx<WST>;

    fn ctx_mut(&mut self) -> &mut Self::Ctx {
        &mut self.ctx
    }
}
