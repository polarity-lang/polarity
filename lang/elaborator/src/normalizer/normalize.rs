use syntax::generic::*;

use super::env::Env;
use super::eval::*;
use super::read_back::*;
use crate::result::*;

pub trait Normalize {
    type Nf;

    fn normalize(&self, prg: &Prg, env: &mut Env) -> Result<Self::Nf, TypeError>;

    fn normalize_in_empty_env(&self, prg: &Prg) -> Result<Self::Nf, TypeError> {
        self.normalize(prg, &mut Env::empty())
    }
}

impl<T> Normalize for T
where
    T: Eval,
    <T as Eval>::Val: ReadBack,
{
    type Nf = <<T as Eval>::Val as ReadBack>::Nf;

    fn normalize(&self, prg: &Prg, env: &mut Env) -> Result<Self::Nf, TypeError> {
        let val = self.eval(prg, env)?;
        val.read_back(prg)
    }
}
