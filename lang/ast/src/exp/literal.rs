use derivative::Derivative;
use pretty::DocAllocator;

use polarity_lang_miette_util::codespan::Span;
use polarity_lang_printer::{Alloc, Builder, Precedence, Print, PrintCfg};

use crate::{
    ContainsMetaVars, Exp, FreeVars, HasSpan, HasType, MetaVar, Occurs, Shift, ShiftRange, Subst,
    Substitutable, Zonk, ZonkError,
    ctx::LevelCtx,
    rename::{Rename, RenameCtx},
};

#[derive(Debug, Clone, PartialEq, Hash, Derivative)]
pub struct Literal {
    /// Source code location
    pub span: Option<Span>,

    /// The kind of literals with its concrete payload
    pub kind: LiteralKind,

    /// The type for literals gets resolved at lowering and will not change
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub inferred_type: Box<Exp>,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum LiteralKind {
    Str { original: String, unescaped: String },
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
    fn shift_in_range<R: ShiftRange>(&mut self, _range: &R, _by: (isize, isize)) {}
}

impl Occurs for Literal {
    fn occurs<F>(&self, _ctx: &mut LevelCtx, _f: &F) -> bool
    where
        F: Fn(&LevelCtx, &Exp) -> bool,
    {
        false
    }
}

impl HasType for Literal {
    fn typ(&self) -> Option<Box<Exp>> {
        Some(self.inferred_type.clone())
    }
}

impl Substitutable for Literal {
    type Target = Literal;

    fn subst(&self, _ctx: &mut LevelCtx, _subst: &Subst) -> Self::Target {
        self.clone()
    }
}

impl Print for Literal {
    fn print_prec<'a>(
        &'a self,
        _cfg: &PrintCfg,
        alloc: &'a Alloc<'a>,
        _prec: Precedence,
    ) -> Builder<'a> {
        let Literal { kind, .. } = self;
        match kind {
            LiteralKind::Str { original, .. } => alloc.text(format!(r#""{}""#, original)),
        }
    }
}

impl Zonk for Literal {
    fn zonk(
        &mut self,
        _meta_vars: &crate::HashMap<MetaVar, crate::MetaVarState>,
    ) -> Result<(), ZonkError> {
        Ok(())
    }
}

impl ContainsMetaVars for Literal {
    fn contains_metavars(&self) -> bool {
        false
    }
}

impl Rename for Literal {
    fn rename_in_ctx(&mut self, _ctx: &mut RenameCtx) {}
}

impl FreeVars for Literal {
    fn free_vars_mut(
        &self,
        _ctx: &LevelCtx,
        _cutoff: usize,
        _fvs: &mut crate::HashSet<crate::Lvl>,
    ) {
    }
}
