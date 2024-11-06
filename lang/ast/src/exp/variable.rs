use codespan::Span;
use derivative::Derivative;
use pretty::DocAllocator;
use printer::{Alloc, Builder, Precedence, Print, PrintCfg};

use crate::{
    ctx::LevelCtx, ContainsMetaVars, HasSpan, HasType, Occurs, Shift, ShiftRange, Substitutable,
    Substitution, Zonk, ZonkError,
};

use super::{Exp, Ident, Idx, Lvl, MetaVar};

/// A bound variable occurrence. The variable is represented
/// using a de-Bruijn index, but we keep the information
/// about the name that was originally annotated in the program.
#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Variable {
    /// Source code location
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub span: Option<Span>,
    /// The de-Bruijn index that is used to represent the
    /// binding structure of terms.
    pub idx: Idx,
    /// The name that was originally annotated in the program
    /// We do not use this information for tracking the binding
    /// structure, but only for prettyprinting code.
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub name: Ident,
    /// Inferred type annotated after elaboration.
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub inferred_type: Option<Box<Exp>>,
}

impl HasSpan for Variable {
    fn span(&self) -> Option<Span> {
        self.span
    }
}

impl From<Variable> for Exp {
    fn from(val: Variable) -> Self {
        Exp::Variable(val)
    }
}

impl Shift for Variable {
    fn shift_in_range<R: ShiftRange>(&mut self, range: &R, by: (isize, isize)) {
        self.idx.shift_in_range(range, by);
        self.inferred_type = None;
    }
}

impl Occurs for Variable {
    fn occurs(&self, ctx: &mut LevelCtx, lvl: Lvl) -> bool {
        let Variable { idx, .. } = self;
        ctx.idx_to_lvl(*idx) == lvl
    }
}

impl HasType for Variable {
    fn typ(&self) -> Option<Box<Exp>> {
        self.inferred_type.clone()
    }
}

impl Substitutable for Variable {
    type Result = Box<Exp>;
    fn subst<S: Substitution>(&self, ctx: &mut LevelCtx, by: &S) -> Self::Result {
        let Variable { span, idx, name, .. } = self;
        match by.get_subst(ctx, ctx.idx_to_lvl(*idx)) {
            Some(exp) => exp,
            None => Box::new(Exp::Variable(Variable {
                span: *span,
                idx: *idx,
                name: name.clone(),
                inferred_type: None,
            })),
        }
    }
}

impl Print for Variable {
    fn print_prec<'a>(
        &'a self,
        cfg: &PrintCfg,
        alloc: &'a Alloc<'a>,
        _prec: Precedence,
    ) -> Builder<'a> {
        let Variable { name, idx, .. } = self;
        if cfg.de_bruijn {
            alloc.text(format!("{name}@{idx}"))
        } else if name.id.is_empty() {
            alloc.text(format!("@{idx}"))
        } else {
            alloc.text(&name.id)
        }
    }
}

impl Zonk for Variable {
    fn zonk(
        &mut self,
        meta_vars: &crate::HashMap<MetaVar, crate::MetaVarState>,
    ) -> Result<(), ZonkError> {
        let Variable { span: _, idx: _, name: _, inferred_type } = self;
        inferred_type.zonk(meta_vars)?;
        Ok(())
    }
}

impl ContainsMetaVars for Variable {
    fn contains_metavars(&self) -> bool {
        let Variable { span: _, idx: _, name: _, inferred_type } = self;

        inferred_type.contains_metavars()
    }
}
