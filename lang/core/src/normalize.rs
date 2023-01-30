use syntax::env::Env;
use syntax::ust;

use super::eval::*;
use super::read_back::*;
use super::result::*;

pub trait Normalize {
    type Nf;

    fn normalize(&self, prg: &ust::Prg, env: &mut Env) -> Result<Self::Nf, NormalizeError>;
}

impl<T> Normalize for T
where
    T: Eval,
    <T as Eval>::Val: ReadBack,
{
    type Nf = <<T as Eval>::Val as ReadBack>::Nf;

    fn normalize(&self, prg: &ust::Prg, env: &mut Env) -> Result<Self::Nf, NormalizeError> {
        // FIXME: Implement error handling
        let val = self.eval(prg, env).unwrap();
        let nf = val.read_back(prg).unwrap();
        Ok(nf)
    }
}
