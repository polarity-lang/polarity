//! Variable context
//!
//! Tracks locally bound variables

use std::rc::Rc;

use crate::normalizer::env::{Env, ToEnv};
use crate::normalizer::normalize::Normalize;
use printer::Print;
use syntax::ast::*;
use syntax::ctx::values::TypeCtx;
use syntax::ctx::{BindContext, Context, LevelCtx};

use crate::result::TypeError;

use super::lookup_table::LookupTable;

#[derive(Debug, Clone)]
pub struct Ctx {
    /// Typing of bound variables
    pub vars: TypeCtx,
    /// Global meta variables and their state
    pub meta_vars: HashMap<MetaVar, MetaVarState>,
    /// Global lookup table for declarations
    pub lookup_table: Rc<LookupTable>,
    /// The program for looking up the expressions when evaluating
    pub module: Rc<Module>,
}

impl Ctx {
    pub fn new(
        meta_vars: HashMap<MetaVar, MetaVarState>,
        lookup_table: LookupTable,
        module: Rc<Module>,
    ) -> Self {
        Self { vars: TypeCtx::empty(), meta_vars, lookup_table: Rc::new(lookup_table), module }
    }
}

pub trait ContextSubstExt: Sized {
    fn subst<S: Substitution>(&mut self, prg: &Module, s: &S) -> Result<(), TypeError>;
}

impl ContextSubstExt for Ctx {
    fn subst<S: Substitution>(&mut self, prg: &Module, s: &S) -> Result<(), TypeError> {
        let env = self.vars.env();
        let levels = self.vars.levels();
        self.map_failable(|nf| {
            let exp = nf.subst(&mut levels.clone(), s);
            let nf = exp.normalize(prg, &mut env.clone())?;
            Ok(nf)
        })
    }
}

impl BindContext for Ctx {
    type Ctx = TypeCtx;

    fn ctx_mut(&mut self) -> &mut Self::Ctx {
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

    pub fn lookup<V: Into<Var> + std::fmt::Debug>(&self, idx: V) -> Rc<Exp> {
        self.vars.lookup(idx).typ
    }

    pub fn levels(&self) -> LevelCtx {
        self.vars.levels()
    }

    pub fn map_failable<E, F>(&mut self, f: F) -> Result<(), E>
    where
        F: Fn(&Rc<Exp>) -> Result<Rc<Exp>, E>,
    {
        self.vars = self.vars.map_failable(f)?;
        Ok(())
    }

    pub fn fork<T, F: FnOnce(&mut Ctx) -> T>(&mut self, f: F) -> T {
        let meta_vars = std::mem::take(&mut self.meta_vars);
        let mut inner_ctx = Ctx {
            vars: self.vars.clone(),
            meta_vars,
            lookup_table: self.lookup_table.clone(),
            module: self.module.clone(),
        };
        let res = f(&mut inner_ctx);
        self.meta_vars = inner_ctx.meta_vars;
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
