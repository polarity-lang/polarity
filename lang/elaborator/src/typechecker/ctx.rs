//! Variable context
//!
//! Tracks locally bound variables

use std::rc::Rc;

use crate::normalizer::env::{Env, ToEnv};
use crate::normalizer::normalize::Normalize;
use ast::ctx::values::TypeCtx;
use ast::ctx::{BindContext, LevelCtx};
use ast::*;
use printer::Print;

use crate::result::TcResult;

use super::type_info_table::TypeInfoTable;
use super::TypeError;

#[derive(Debug, Clone)]
pub struct Ctx {
    /// Typing of bound variables
    pub vars: TypeCtx,
    /// Global meta variables and their state
    pub meta_vars: HashMap<MetaVar, MetaVarState>,
    /// Global lookup table for declarations
    pub type_info_table: Rc<TypeInfoTable>,
    /// The program for looking up the expressions when evaluating
    pub module: Rc<Module>,
    /// Declarations that were lifted during typechecking
    pub lifted_decls: Vec<Decl>,
}

impl Ctx {
    pub fn new(
        meta_vars: HashMap<MetaVar, MetaVarState>,
        type_info_table: TypeInfoTable,
        module: Rc<Module>,
    ) -> Self {
        Self {
            vars: TypeCtx::empty(),
            meta_vars,
            type_info_table: Rc::new(type_info_table),
            module,
            lifted_decls: Vec::new(),
        }
    }
}
pub trait ContextSubstExt: Sized {
    fn subst<S: Substitution>(&mut self, type_info_table: &Rc<TypeInfoTable>, s: &S) -> TcResult
    where
        S::Err: Into<TypeError>;
}

impl ContextSubstExt for Ctx {
    fn subst<S: Substitution>(&mut self, type_info_table: &Rc<TypeInfoTable>, s: &S) -> TcResult
    where
        S::Err: Into<TypeError>,
    {
        let env = self.vars.env();
        let levels = self.vars.levels();
        self.map_failable(|nf| {
            let exp = nf.subst(&mut levels.clone(), s).map_err(Into::into)?;
            let nf = exp.normalize(type_info_table, &mut env.clone())?;
            Ok(nf)
        })
    }
}

impl BindContext for Ctx {
    type Content = Box<Exp>;

    fn ctx_mut(&mut self) -> &mut TypeCtx {
        &mut self.vars
    }
}

impl ToEnv for Ctx {
    fn env(&self) -> Env {
        self.vars.env()
    }
}

impl Ctx {
    pub fn len(&self) -> usize {
        self.vars.len()
    }

    pub fn is_empty(&self) -> bool {
        self.vars.is_empty()
    }

    pub fn lookup<V: Into<Var> + std::fmt::Debug>(&self, idx: V) -> Box<Exp> {
        self.vars.lookup(idx).content
    }

    pub fn levels(&self) -> LevelCtx {
        self.vars.levels()
    }

    pub fn map_failable<E, F>(&mut self, f: F) -> Result<(), E>
    where
        F: Fn(&Exp) -> Result<Box<Exp>, E>,
    {
        self.vars = self.vars.map_failable(f)?;
        Ok(())
    }

    pub fn fork<T, F: FnOnce(&mut Ctx) -> T>(&mut self, f: F) -> T {
        let meta_vars = std::mem::take(&mut self.meta_vars);
        let lifted_decls = std::mem::take(&mut self.lifted_decls);
        let mut inner_ctx = Ctx {
            vars: self.vars.clone(),
            meta_vars,
            type_info_table: self.type_info_table.clone(),
            module: self.module.clone(),
            lifted_decls,
        };
        let res = f(&mut inner_ctx);
        self.meta_vars = inner_ctx.meta_vars;
        self.lifted_decls = inner_ctx.lifted_decls;
        res
    }
}

impl Print for Ctx {
    fn print<'a>(
        &'a self,
        cfg: &printer::PrintCfg,
        alloc: &'a printer::Alloc<'a>,
    ) -> printer::Builder<'a> {
        self.vars.print(cfg, alloc)
    }
}
