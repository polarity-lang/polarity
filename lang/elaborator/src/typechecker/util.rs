use std::rc::Rc;

use log::trace;

use ast::ctx::LevelCtx;
use ast::*;
use printer::types::Print;

use crate::unifier::{constraints::Constraint, unify::unify};

use super::TypeError;

// Checks whether the codata type contains destructors with a self parameter
pub fn uses_self(codata: &Codata) -> Result<bool, TypeError> {
    for dtor in &codata.dtors {
        let mut ctx = LevelCtx::from(vec![dtor.params.len(), 1]);
        if dtor.ret_typ.occurs(&mut ctx, Lvl { fst: 1, snd: 0 }) {
            return Ok(true);
        }
    }
    Ok(false)
}

pub fn convert(
    ctx: LevelCtx,
    meta_vars: &mut HashMap<MetaVar, MetaVarState>,
    this: Rc<Exp>,
    other: &Rc<Exp>,
) -> Result<(), TypeError> {
    trace!("{} =? {}", this.print_trace(), other.print_trace());
    // Convertibility is checked using the unification algorithm.
    let constraint: Constraint = Constraint::Equality { lhs: this.clone(), rhs: other.clone() };
    let res = unify(ctx, meta_vars, constraint, true)?;
    match res {
        crate::unifier::dec::Dec::Yes(_) => Ok(()),
        crate::unifier::dec::Dec::No(_) => Err(TypeError::not_eq(this.clone(), other.clone())),
    }
}

pub trait ExpectTypApp {
    fn expect_typ_app(&self) -> Result<TypCtor, TypeError>;
}

impl ExpectTypApp for Rc<Exp> {
    fn expect_typ_app(&self) -> Result<TypCtor, TypeError> {
        match &**self {
            Exp::TypCtor(TypCtor { span, name, args }) => {
                Ok(TypCtor { span: *span, name: name.clone(), args: args.clone() })
            }
            _ => Err(TypeError::expected_typ_app(self.clone())),
        }
    }
}
