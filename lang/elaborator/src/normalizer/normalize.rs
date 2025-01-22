use std::rc::Rc;

use crate::normalizer::val::ReadBack;
use crate::{result::*, TypeInfoTable};

use super::env::Env;
use super::eval::*;

pub trait Normalize {
    type Nf;

    fn normalize(&self, info_table: &Rc<TypeInfoTable>, env: &mut Env) -> TcResult<Self::Nf>;

    fn normalize_in_empty_env(&self, info_table: &Rc<TypeInfoTable>) -> TcResult<Self::Nf> {
        self.normalize(info_table, &mut Env::empty())
    }
}

impl<T> Normalize for T
where
    T: Eval,
    <T as Eval>::Val: ReadBack,
{
    type Nf = <<T as Eval>::Val as ReadBack>::Nf;

    fn normalize(&self, info_table: &Rc<TypeInfoTable>, env: &mut Env) -> TcResult<Self::Nf> {
        let val = self.eval(info_table, env)?;
        val.read_back(info_table)
    }
}
