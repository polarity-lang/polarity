use codespan::Span;
use log::trace;

use ast::ctx::LevelCtx;
use ast::*;
use printer::types::Print;

use crate::conversion_checking::{constraints::Constraint, unify::unify};

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
    this: Box<Exp>,
    other: &Exp,
    while_elaborating_span: &Option<Span>,
) -> Result<(), TypeError> {
    trace!("{} =? {}", this.print_trace(), other.print_trace());
    // Convertibility is checked using the unification algorithm.
    let constraint: Constraint =
        Constraint::Equality { lhs: this.clone(), rhs: Box::new(other.clone()) };
    let res = unify(ctx, meta_vars, constraint, true, while_elaborating_span)?;
    match res {
        crate::conversion_checking::dec::Dec::Yes(_) => Ok(()),
        crate::conversion_checking::dec::Dec::No(_) => {
            Err(TypeError::not_eq(&this, other, while_elaborating_span))
        }
    }
}

pub trait ExpectTypApp {
    fn expect_typ_app(&self) -> Result<TypCtor, TypeError>;
}

impl ExpectTypApp for Exp {
    fn expect_typ_app(&self) -> Result<TypCtor, TypeError> {
        match self {
            Exp::TypCtor(TypCtor { span, name, args }) => {
                Ok(TypCtor { span: *span, name: name.clone(), args: args.clone() })
            }
            _ => Err(TypeError::expected_typ_app(self)),
        }
    }
}
