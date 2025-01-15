//! This module defines the language of constraints that can be solved by the constraint solver.
use ast::{Args, Exp};
use printer::Print;

/// A constraint that can be solved by the constraint solver.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Constraint {
    /// An equality constraint between two expressions.
    Equality { lhs: Box<Exp>, rhs: Box<Exp> },
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
