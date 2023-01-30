//! Variable context
//!
//! Tracks locally bound variables

use std::rc::Rc;

use data::HashMap;
use syntax::common::*;
use syntax::ctx::values::TypeCtx;
use syntax::ust;

use crate::eval::Eval;
use crate::read_back::ReadBack;
use crate::TypeError;

pub type Ctx = TypeCtx;

pub struct NameGen {
    /// Count unnamed xmatches to generate names for them
    pub names: HashMap<Ident, usize>,
}

pub fn empty_name_gen() -> NameGen {
    NameGen { names: HashMap::default() }
}

pub trait ContextSubstExt: Sized {
    fn subst<S: Substitution<Rc<ust::Exp>>>(
        &self,
        prg: &ust::Prg,
        s: &S,
    ) -> Result<Self, TypeError>;
}

impl ContextSubstExt for Ctx {
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
