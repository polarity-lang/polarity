//! Variable context
//!
//! Tracks locally bound variables

use std::rc::Rc;

use crate::normalizer::env::{Env, ToEnv};
use crate::normalizer::normalize::Normalize;
use ast::ctx::values::TypeCtx;
use ast::ctx::{BindContext, Context, LevelCtx};
use ast::*;
use printer::Print;

use crate::result::TypeError;

use super::type_info_table::TypeInfoTable;

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
        }
    }

    /// Check that there are no unresolved metavariables that remain after typechecking.
    pub fn check_metavars_solved(&self) -> Result<(), TypeError> {
        let mut unsolved: HashSet<MetaVar> = HashSet::default();
        for (var, state) in self.meta_vars.iter() {
            // We only have to throw an error for unsolved metavars which were either
            // inserted or are holes `_` which must be solved
            // Unsolved metavariables that correspond to typed holes `?` do not lead
            // to an error.
            if !state.is_solved() && var.must_be_solved() {
                unsolved.insert(*var);
            }
        }

        if !unsolved.is_empty() {
            Err(TypeError::UnresolvedMetas { message: format!("{:?}", unsolved) })
        } else {
            Ok(())
        }
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

    pub fn lookup<V: Into<Var> + std::fmt::Debug>(&self, idx: V) -> Box<Exp> {
        self.vars.lookup(idx).typ
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
        let mut inner_ctx = Ctx {
            vars: self.vars.clone(),
            meta_vars,
            type_info_table: self.type_info_table.clone(),
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
