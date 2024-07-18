//! This module defines the language of constraints that can be solve by the constraint solver.
use std::rc::Rc;

use printer::Print;
use syntax::ast::Exp;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Constraint {
    Equality { lhs: Rc<Exp>, rhs: Rc<Exp> },
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
        }
    }
}
