//! Variable context
//!
//! Tracks locally bound variables

use std::rc::Rc;

use normalizer::env::Env;
use normalizer::env::ToEnv;
use normalizer::eval::Eval;
use normalizer::read_back::ReadBack;
use printer::Print;
use syntax::common::*;
use syntax::ctx::values::TypeCtx;
use syntax::ctx::{Context, HasContext, LevelCtx};
use syntax::nf::Nf;
use syntax::ust;

use crate::ng::NameGen;
use crate::TypeError;

pub struct Ctx {
    /// Typing of bound variables
    vars: TypeCtx,
    /// Name generator for (co)match labels
    ng: NameGen,
}

impl Default for Ctx {
    fn default() -> Self {
        Self { vars: TypeCtx::empty(), ng: Default::default() }
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
            let val = exp.eval(prg, &mut env.clone())?;
            Ok(val.read_back(prg)?)
        })
    }
}

impl HasContext for Ctx {
    type Ctx = TypeCtx;

    fn ctx_mut(&mut self) -> &mut Self::Ctx {
        &mut self.vars
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
        self.vars.lookup(idx)
    }

    pub fn env(&self) -> Env {
        self.vars.env()
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
        let mut inner_ctx = Ctx { vars: self.vars.clone(), ng: std::mem::take(&mut self.ng) };
        let out = f(&mut inner_ctx);
        self.ng = inner_ctx.ng;
        out
    }

    pub fn fresh_label(&mut self, type_name: &str, prg: &ust::Prg) -> Ident {
        self.ng.fresh_label(type_name, prg)
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
