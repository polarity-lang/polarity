use polarity_lang_miette_util::codespan::Span;
use polarity_lang_printer::{Alloc, Builder, Precedence, Print, PrintCfg};

use crate::{
    ContainsMetaVars, Exp, FreeVars, HasSpan, HasType, MetaVar, Occurs, Shift, ShiftRange, Subst,
    Substitutable, Zonk, ZonkError,
    ctx::LevelCtx,
    rename::{Rename, RenameCtx},
};

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Literal {
    /// Source code location
    pub span: Option<Span>,

    pub kind: LiteralKind,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum LiteralKind {
    Str(String),
}

impl HasSpan for Literal {
    fn span(&self) -> Option<Span> {
        self.span
    }
}

impl From<Literal> for Exp {
    fn from(val: Literal) -> Self {
        Exp::Literal(val)
    }
}

impl Shift for Literal {
    fn shift_in_range<R: ShiftRange>(&mut self, range: &R, by: (isize, isize)) {
        todo!()
    }
}

impl Occurs for Literal {
    fn occurs<F>(&self, ctx: &mut LevelCtx, f: &F) -> bool
    where
        F: Fn(&LevelCtx, &Exp) -> bool,
    {
        todo!()
    }
}

impl HasType for Literal {
    fn typ(&self) -> Option<Box<Exp>> {
        todo!()
    }
}

impl Substitutable for Literal {
    type Target = Literal;

    fn subst(&self, ctx: &mut LevelCtx, subst: &Subst) -> Self::Target {
        todo!()
    }
}

impl Print for Literal {
    fn print_prec<'a>(
        &'a self,
        cfg: &PrintCfg,
        alloc: &'a Alloc<'a>,
        _prec: Precedence,
    ) -> Builder<'a> {
        todo!()
    }
}

impl Zonk for Literal {
    fn zonk(
        &mut self,
        meta_vars: &crate::HashMap<MetaVar, crate::MetaVarState>,
    ) -> Result<(), ZonkError> {
        todo!()
    }
}

impl ContainsMetaVars for Literal {
    fn contains_metavars(&self) -> bool {
        todo!()
    }
}

impl Rename for Literal {
    fn rename_in_ctx(&mut self, ctx: &mut RenameCtx) {
        todo!()
    }
}

impl FreeVars for Literal {
    fn free_vars_mut(&self, ctx: &LevelCtx, cutoff: usize, fvs: &mut crate::HashSet<crate::Lvl>) {
        todo!()
    }
}
