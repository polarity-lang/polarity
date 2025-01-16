//! This module defines the language of constraints that can be solved by the constraint solver.
use ast::{ctx::values::TypeCtx, Args, Exp};
use derivative::Derivative;
use printer::Print;

/// A constraint that can be solved by the constraint solver.
#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub enum Constraint {
    /// An equality constraint between two expressions under the same context.
    /// ctx |- lhs = rhs
    Equality {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        ctx: TypeCtx,
        lhs: Box<Exp>,
        rhs: Box<Exp>,
    },
    /// An equality constraint between two argument lists under the same context.
    /// ctx |- lhs = rhs
    EqualityArgs {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        ctx: TypeCtx,
        lhs: Args,
        rhs: Args,
    },
}

impl Print for Constraint {
    fn print<'a>(
        &'a self,
        cfg: &printer::PrintCfg,
        alloc: &'a printer::Alloc<'a>,
    ) -> printer::Builder<'a> {
        match self {
            Constraint::Equality { ctx, lhs, rhs } => ctx
                .print(cfg, alloc)
                .append(" |- ")
                .append(lhs.print(cfg, alloc))
                .append(" = ")
                .append(rhs.print(cfg, alloc)),
            Constraint::EqualityArgs { ctx, lhs, rhs } => ctx
                .print(cfg, alloc)
                .append(" |- ")
                .append(lhs.print(cfg, alloc))
                .append(" = ")
                .append(rhs.print(cfg, alloc)),
        }
    }
}
