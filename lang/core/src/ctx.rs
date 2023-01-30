//! Variable context
//!
//! Tracks locally bound variables

use std::rc::Rc;

use syntax::common::*;
use syntax::ctx::values::TypeCtx;
use syntax::ctx::{Context, HasContext};
use syntax::ust;

use crate::eval::Eval;
use crate::read_back::ReadBack;
use crate::TypeError;

//pub type Ctx = TypeCtx;

pub struct Ctx {
    pub vars: TypeCtx,
}

impl Ctx {
    pub fn empty() -> Self {
        Self { vars: TypeCtx::empty() }
    }
    pub fn new(vars: TypeCtx) -> Self {
        Self { vars }
    }
}

impl HasContext for Ctx {
    type Ctx = TypeCtx;

    fn ctx_mut(&mut self) -> &mut Self::Ctx {
        &mut self.vars
    }
}

pub trait ContextSubstExt: Sized {
    fn subst<S: Substitution<Rc<ust::Exp>>>(
        &self,
        prg: &ust::Prg,
        s: &S,
    ) -> Result<Self, TypeError>;
}

impl ContextSubstExt for TypeCtx {
    fn subst<S: Substitution<Rc<ust::Exp>>>(
        &self,
        prg: &ust::Prg,
        s: &S,
    ) -> Result<Self, TypeError> {
        self.map_failable(|val| {
            let nf = val.read_back(prg)?;
            let exp = nf.forget().subst(&mut self.levels(), s);
            exp.eval(prg, &mut self.env()).map_err(Into::into)
        })
    }
}
