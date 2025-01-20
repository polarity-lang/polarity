use derivative::Derivative;
use miette_util::codespan::Span;
use printer::{theme::ThemeExt, tokens::TYPE, Alloc, Builder, Precedence, Print, PrintCfg};

use crate::{
    ctx::LevelCtx, ContainsMetaVars, HasSpan, HasType, Occurs, Shift, ShiftRange, Substitutable,
    Substitution, Zonk, ZonkError,
};

use super::{Exp, Lvl, MetaVar};

/// The impredicative type universe "Type" is used
/// for typing data and codata types. I.e. we have
/// - `Nat : Type`
/// - `Stream(Nat) : Type`
/// - `Type : Type`
#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct TypeUniv {
    /// Source code location
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub span: Option<Span>,
}

impl TypeUniv {
    pub fn new() -> TypeUniv {
        TypeUniv { span: None }
    }
}

impl HasSpan for TypeUniv {
    fn span(&self) -> Option<Span> {
        self.span
    }
}

impl From<TypeUniv> for Exp {
    fn from(val: TypeUniv) -> Self {
        Exp::TypeUniv(val)
    }
}

impl Default for TypeUniv {
    fn default() -> Self {
        Self::new()
    }
}

impl Shift for TypeUniv {
    fn shift_in_range<R: ShiftRange>(&mut self, _range: &R, _by: (isize, isize)) {}
}

impl Occurs for TypeUniv {
    fn occurs(&self, _ctx: &mut LevelCtx, _lvl: Lvl) -> bool {
        false
    }
}

impl HasType for TypeUniv {
    fn typ(&self) -> Option<Box<Exp>> {
        Some(Box::new(TypeUniv::new().into()))
    }
}

impl Substitutable for TypeUniv {
    type Result = TypeUniv;

    fn subst<S: Substitution>(&self, _ctx: &mut LevelCtx, _by: &S) -> Self::Result {
        let TypeUniv { span } = self;
        TypeUniv { span: *span }
    }
}

impl Print for TypeUniv {
    fn print_prec<'a>(
        &'a self,
        _cfg: &PrintCfg,
        alloc: &'a Alloc<'a>,
        _prec: Precedence,
    ) -> Builder<'a> {
        alloc.keyword(TYPE)
    }
}

/// Implement Zonk for TypeUniv
impl Zonk for TypeUniv {
    fn zonk(
        &mut self,
        _meta_vars: &crate::HashMap<MetaVar, crate::MetaVarState>,
    ) -> Result<(), ZonkError> {
        // TypeUniv has no fields that require zonking
        Ok(())
    }
}

impl ContainsMetaVars for TypeUniv {
    fn contains_metavars(&self) -> bool {
        false
    }
}
