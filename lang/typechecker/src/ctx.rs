//! Variable context
//!
//! Tracks locally bound variables

use std::rc::Rc;

use normalizer::env::{Env, ToEnv};
use normalizer::normalize::Normalize;
use printer::Print;
use syntax::common::*;
use syntax::ctx::values::TypeCtx;
use syntax::ctx::{BindContext, Context, LevelCtx};
use syntax::nf::Nf;
use syntax::ust;

use crate::TypeError;

#[derive(Debug, Clone)]
pub struct Ctx {
    /// Typing of bound variables
    pub vars: TypeCtx,
}

impl Default for Ctx {
    fn default() -> Self {
        Self { vars: TypeCtx::empty() }
    }
}

pub trait ContextSubstExt: Sized {
    fn subst<S: Substitution<Rc<ust::Exp>>>(
        &mut self,
        prg: &ust::Prg,
        s: &S,
    ) -> Result<(), TypeError>;
}

impl ContextSubstExt for Ctx {
    fn subst<S: Substitution<Rc<ust::Exp>>>(
        &mut self,
        prg: &ust::Prg,
        s: &S,
    ) -> Result<(), TypeError> {
        let env = self.vars.env();
        let levels = self.vars.levels();
        self.map_failable(|nf| {
            let exp = nf.forget().subst(&mut levels.clone(), s);
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

    pub fn lookup<V: Into<Var> + std::fmt::Debug>(&self, idx: V) -> Rc<Nf> {
        self.vars.lookup(idx).typ
    }

    pub fn levels(&self) -> LevelCtx {
        self.vars.levels()
    }

    pub fn map_failable<E, F>(&mut self, f: F) -> Result<(), E>
    where
        F: Fn(&Rc<Nf>) -> Result<Rc<Nf>, E>,
    {
        self.vars = self.vars.map_failable(f)?;
        Ok(())
    }

    pub fn fork<T, F: FnOnce(&mut Ctx) -> T>(&mut self, f: F) -> T {
        let mut inner_ctx = Ctx { vars: self.vars.clone() };
        f(&mut inner_ctx)
    }
}

impl<'a> Print<'a> for Ctx {
    fn print(
        &'a self,
        cfg: &printer::PrintCfg,
        alloc: &'a printer::Alloc<'a>,
    ) -> printer::Builder<'a> {
        self.vars.print(cfg, alloc)
    }
}
