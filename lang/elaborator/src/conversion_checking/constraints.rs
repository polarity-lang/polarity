//! This module defines the language of constraints that can be solved by the constraint solver.
use derivative::Derivative;
use polarity_lang_ast::{Args, Exp, ctx::values::TypeCtx};
use polarity_lang_printer::Print;

/// A constraint that can be solved by the constraint solver.
#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub enum Constraint<'a> {
    /// An equality constraint between two expressions under the same context.
    /// ctx |- lhs = rhs
    Equality {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        ctx: &'a TypeCtx,
        lhs: Box<Exp>,
        rhs: Box<Exp>,
    },
    /// An equality constraint between two argument lists under the same context.
    /// ctx |- lhs = rhs
    EqualityArgs {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        ctx: &'a TypeCtx,
        lhs: Args,
        rhs: Args,
    },
}

impl Print for Constraint<'_> {
    fn print<'a>(
        &'a self,
        cfg: &polarity_lang_printer::PrintCfg,
        alloc: &'a polarity_lang_printer::Alloc<'a>,
    ) -> polarity_lang_printer::Builder<'a> {
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
