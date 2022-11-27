use std::rc::Rc;

use data::HashSet;
use renaming::Rename;
use syntax::ast::forget::Forget;
use syntax::ast::fv::FreeVarsExt;
use syntax::ast::fv::FreeVarsResult;
use syntax::ast::*;
use syntax::common::*;
use syntax::ctx::*;
use syntax::tst;
use syntax::tst::TST;

#[derive(Debug)]
pub struct Lift {
    /// The type name that should be lifted
    name: String,
    /// List of new top-level declarations that got created in the lifting process
    new_decls: Vec<tst::Decl>,
    /// Typing context
    ctx: TypeCtx<TST>,
    /// Current declaration being visited for lifting
    curr_decl: Ident,
    /// List of declarations that got modified in the lifting process
    modified_decls: HashSet<Ident>,
}

/// Result of lifting
pub struct LiftResult {
    /// The resulting program
    pub prg: tst::Prg,
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

    pub fn run(mut self, prg: tst::Prg) -> LiftResult {
        // Perform the lifting process
        let mut prg_out = prg.map(&mut self);

        // Update program accordingly
        let mut impl_block = prg_out.decls.source.get_or_add_impl_block(self.name.clone());
        impl_block.set_defs(self.new_decls.iter().map(|decl| decl.name().clone()));

        match prg_out.decls.map.get_mut(&self.name).unwrap() {
            Decl::Codata(codata) => {
                codata.impl_block = Some(Impl {
                    // FIXME: might not exist
                    info: codata.impl_block.as_ref().unwrap().info.clone(),
                    name: codata.name.clone(),
                    defs: self.new_decls.iter().map(|def| def.name().clone()).collect(),
                })
            }
            Decl::Data(data) => {
                data.impl_block = Some(Impl {
                    // FIXME: might not exist
                    info: data.impl_block.as_ref().unwrap().info.clone(),
                    name: data.name.clone(),
                    defs: self.new_decls.iter().map(|def| def.name().clone()).collect(),
                })
            }
            _ => unreachable!(),
        }

        let decls_iter = self.new_decls.into_iter().map(|decl| (decl.name().clone(), decl));
        prg_out.decls.map.extend(decls_iter);

        // Rename program to ensure proper variable naming for lifted definitions
        let prg_out = prg_out.rename();

        LiftResult { prg: prg_out, modified_decls: self.modified_decls }
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

impl Mapper<TST> for Lift {
    fn enter_decl(&mut self, decl: &Decl<TST>) {
        self.set_curr_decl(decl.name().clone());
    }

    fn map_exp_match(
        &mut self,
        info: tst::TypeAppInfo,
        name: Ident,
        on_exp: Rc<tst::Exp>,
        in_typ: tst::Typ,
        body: tst::Match,
    ) -> tst::Exp {
        // Only lift local matches for the specified type
        if info.typ.name != self.name {
            return id().map_exp_match(info, name, on_exp, in_typ, body);
        }

        self.mark_modified();

        // Collect the free variables in the match,
        // the type of the scrutinee as well as the return type of the match
        // Build a telescope of the types of the lifted variables
        let FreeVarsResult { telescope, subst, args } = body
            .free_vars(&mut self.ctx)
            .union(info.typ.free_vars(&mut self.ctx))
            .union(in_typ.as_exp().free_vars(&mut self.ctx))
            .telescope();

        // Substitute the new parameters for the free variables
        let body = body.subst(&mut self.ctx.levels(), &subst);
        let on_typ = info.typ.subst(&mut self.ctx.levels(), &subst);
        let def_in_typ = in_typ.as_exp().subst(&mut self.ctx.levels(), &subst);

        // Build the new top-level definition
        let def = tst::Def {
            info: tst::Info::empty(),
            name: name.clone(),
            params: telescope,
            on_typ,
            in_typ: def_in_typ,
            body,
        };

        self.new_decls.push(Decl::Def(def));

        // Replace the match by a destructor call of the new top-level definition
        tst::Exp::Dtor {
            info: tst::TypeInfo { typ: in_typ.as_exp().forget(), span: Default::default() },
            exp: on_exp,
            name,
            args,
        }
    }

    fn map_exp_comatch(
        &mut self,
        info: tst::TypeAppInfo,
        name: Ident,
        body: tst::Comatch,
    ) -> tst::Exp {
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
        let codef = tst::Codef {
            info: tst::Info::empty(),
            name: name.clone(),
            params: telescope,
            typ,
            body,
        };

        self.new_decls.push(Decl::Codef(codef));

        // Replace the comatch by a call of the new top-level definition
        tst::Exp::Ctor {
            info: tst::TypeInfo {
                typ: Rc::new(info.typ.to_exp().forget()),
                span: Default::default(),
            },
            name,
            args,
        }
    }

    fn map_telescope<X, I, F1, F2>(&mut self, params: I, f_acc: F1, f_inner: F2) -> X
    where
        I: IntoIterator<Item = Param<TST>>,
        F1: Fn(&mut Self, Param<TST>) -> Param<TST>,
        F2: FnOnce(&mut Self, Telescope<TST>) -> X,
    {
        self.ctx_map_telescope(params, f_acc, f_inner)
    }

    fn map_telescope_inst<X, I, F1, F2>(&mut self, params: I, f_acc: F1, f_inner: F2) -> X
    where
        I: IntoIterator<Item = ParamInst<TST>>,
        F1: Fn(&mut Self, ParamInst<TST>) -> ParamInst<TST>,
        F2: FnOnce(&mut Self, TelescopeInst<TST>) -> X,
    {
        self.ctx_map_telescope_inst(params, f_acc, f_inner)
    }
}

impl HasContext for Lift {
    type Ctx = TypeCtx<TST>;

    fn ctx_mut(&mut self) -> &mut Self::Ctx {
        &mut self.ctx
    }
}
