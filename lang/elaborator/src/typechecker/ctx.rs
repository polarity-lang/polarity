//! Variable context
//!
//! Tracks locally bound variables

use std::rc::Rc;

use crate::normalizer::env::{Env, ToEnv};
use crate::normalizer::normalize::Normalize;
use ast::ctx::values::{Binder, Binding, TypeCtx};
use ast::ctx::{BindContext, LevelCtx};
use ast::*;
use printer::Print;

use crate::result::TcResult;

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
}
pub trait ContextSubstExt: Sized {
    fn subst(&mut self, type_info_table: &Rc<TypeInfoTable>, subst: &Subst) -> TcResult;
}

impl ContextSubstExt for Ctx {
    fn subst(&mut self, type_info_table: &Rc<TypeInfoTable>, subst: &Subst) -> TcResult {
        let env = self.vars.env();
        let levels = self.vars.levels();

        self.vars.bound = self
            .vars
            .bound
            .iter()
            .enumerate()
            .map(|(fst, stack)| {
                stack
                    .iter()
                    .enumerate()
                    .map(|(snd, Binder { name, content: binding })| {
                        let lvl = Lvl { fst, snd };
                        let mut binding = binding.subst_new(&levels.clone(), subst);
                        if binding.val.is_none() {
                            if let Some(val) = subst.hm.get(&lvl) {
                                binding.val = Some(ctx::values::BoundValue::PatternMatching {
                                    val: Box::new(val.clone()),
                                })
                            }
                        }
                        binding.typ = binding.typ.normalize(type_info_table, &mut env.clone())?;
                        Ok(Binder { name: name.clone(), content: binding })
                    })
                    .collect()
            })
            .collect::<TcResult<Vec<_>>>()?;

        Ok(())
    }
}

impl BindContext for Ctx {
    type Content = Binding;

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
        self.vars.lookup(idx).content.typ
    }

    pub fn levels(&self) -> LevelCtx {
        self.vars.levels()
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
