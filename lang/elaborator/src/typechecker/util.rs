use std::rc::Rc;

use log::trace;

use printer::types::Print;
use syntax::{ast::*, ctx::LevelCtx};

use crate::unifier::{constraints::Constraint, unify::unify};

use super::TypeError;

// Checks whether the codata type contains destructors with a self parameter
pub fn uses_self(prg: &Module, codata: &Codata) -> Result<bool, TypeError> {
    for dtor_name in &codata.dtors {
        let dtor = prg.dtor(dtor_name, None)?;
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
    trace!("{} =? {}", this.print_to_colored_string(None), other.print_to_colored_string(None));
    // Convertibility is checked using the unification algorithm.
    let eqn: Constraint = Constraint { lhs: this.clone(), rhs: other.clone() };
    let eqns: Vec<Constraint> = vec![eqn];
    let res = unify(ctx, meta_vars, eqns, true)?;
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
