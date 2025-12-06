use derivative::Derivative;
use pretty::DocAllocator;

use polarity_lang_miette_util::codespan::Span;
use polarity_lang_printer::{Alloc, Builder, Precedence, Print, PrintCfg};

use super::{Exp, Idx, MetaVar, VarBound};
use crate::{
    ContainsMetaVars, FreeVars, HasSpan, HasType, Occurs, Shift, ShiftRange, Subst, Substitutable,
    VarBind, Zonk, ZonkError,
    ctx::LevelCtx,
    rename::{Rename, RenameCtx},
};

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
    pub name: VarBound,
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
    fn occurs<F>(&self, _ctx: &mut LevelCtx, _f: &F) -> bool
    where
        F: Fn(&LevelCtx, &Exp) -> bool,
    {
        false
    }
}

impl HasType for Variable {
    fn typ(&self) -> Option<Box<Exp>> {
        self.inferred_type.clone()
    }
}

impl Substitutable for Variable {
    type Target = Box<Exp>;
    fn subst(&self, ctx: &mut LevelCtx, subst: &Subst) -> Self::Target {
        let Variable { span, idx, name, .. } = self;
        let lvl = ctx.idx_to_lvl(*idx);
        match subst.map.get(&lvl) {
            None => Box::new(Exp::Variable(Variable {
                span: *span,
                idx: *idx,
                name: name.clone(),
                inferred_type: None,
            })),
            Some(exp) => Box::new(exp.clone()),
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

impl Rename for Variable {
    fn rename_in_ctx(&mut self, ctx: &mut RenameCtx) {
        let name = match ctx.binders.lookup(self.idx).name {
            VarBind::Var { id, .. } => id,
            VarBind::Wildcard { .. } => {
                // Currently, any wildcards are replaced by named variables when binding them to the context.
                // Therefore this case is unreachable.
                // In the future, we may want to allow wildcards to survive renaming.
                unreachable!()
            }
        };
        self.name = VarBound::from_string(&name);
        self.inferred_type.rename_in_ctx(ctx);
    }
}

impl FreeVars for Variable {
    fn free_vars_mut(&self, ctx: &LevelCtx, cutoff: usize, fvs: &mut crate::HashSet<crate::Lvl>) {
        let Variable { span: _, idx, name: _, inferred_type: _ } = self;

        if idx.fst >= cutoff {
            let idx = Idx { fst: idx.fst - cutoff, snd: idx.snd };
            fvs.extend([ctx.idx_to_lvl(idx)]);
        }
    }
}
