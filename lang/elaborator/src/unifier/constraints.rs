//! This module defines the language of constraints that can be solved by the constraint solver.
use std::rc::Rc;

use printer::Print;
use syntax::ast::{Args, Exp};

/// A constraint that can be solved by the constraint solver.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Constraint {
    /// An equality constraint between two expressions.
    Equality { lhs: Rc<Exp>, rhs: Rc<Exp> },
    /// An equality constraint between two argument lists.
    EqualityArgs { lhs: Args, rhs: Args },
}

impl Print for Constraint {
    fn print<'a>(
        &'a self,
        cfg: &printer::PrintCfg,
        alloc: &'a printer::Alloc<'a>,
    ) -> printer::Builder<'a> {
        match self {
            Constraint::Equality { lhs, rhs } => {
                lhs.print(cfg, alloc).append(" = ").append(rhs.print(cfg, alloc))
            }
            Constraint::EqualityArgs { lhs, rhs } => {
                lhs.print(cfg, alloc).append(" = ").append(rhs.print(cfg, alloc))
            }
        }
    }
}
