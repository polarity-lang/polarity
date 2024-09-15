use ast::ctx::GenericCtx;
use ast::*;

use super::env::Env;
use super::eval::*;
use crate::normalizer::val::ReadBack;
use crate::result::*;

pub trait Normalize {
    type Nf;

    fn normalize(&self, prg: &Module, env: &mut Env) -> Result<Self::Nf, TypeError>;

    fn normalize_in_empty_env(&self, prg: &Module) -> Result<Self::Nf, TypeError> {
        self.normalize(prg, &mut GenericCtx::empty().into())
    }
}

impl<T> Normalize for T
where
    T: Eval,
    <T as Eval>::Val: ReadBack,
{
    type Nf = <<T as Eval>::Val as ReadBack>::Nf;

    fn normalize(&self, prg: &Module, env: &mut Env) -> Result<Self::Nf, TypeError> {
        let val = self.eval(prg, env)?;
        val.read_back(prg)
    }
}
