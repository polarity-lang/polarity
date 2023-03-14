use crate::env::Env;
use syntax::ust;

use super::eval::*;
use super::read_back::*;
use super::result::*;

pub trait Normalize {
    type Nf;

    fn normalize(&self, prg: &ust::Prg, env: &mut Env) -> Result<Self::Nf, EvalError>;
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
