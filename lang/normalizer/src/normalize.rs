use super::eval::*;
use super::read_back::*;
use super::result::*;
use crate::env::Env;
use syntax::ctx::Context;
use syntax::ust;

pub trait Normalize {
    type Nf;

    fn normalize(&self, prg: &ust::Prg, env: &mut Env) -> Result<Self::Nf, EvalError>;

    fn normalize_in_empty_env(&self, prg: &ust::Prg) -> Result<Self::Nf, EvalError> {
        self.normalize(prg, &mut Env::empty())
    }
}

impl<T> Normalize for T
where
    T: Eval,
    <T as Eval>::Val: ReadBack,
{
    type Nf = <<T as Eval>::Val as ReadBack>::Nf;

    fn normalize(&self, prg: &ust::Prg, env: &mut Env) -> Result<Self::Nf, EvalError> {
        let val = self.eval(prg, env)?;
        val.read_back(prg)
    }
}
