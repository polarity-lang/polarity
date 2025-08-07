use derivative::Derivative;
use miette_util::codespan::Span;
use printer::{Alloc, Builder, Precedence, Print, PrintCfg, theme::ThemeExt, tokens::TYPE};

use crate::{
    ContainsMetaVars, FreeVars, HasSpan, HasType, Inline, IsWHNF, MachineState, Shift, ShiftRange,
    Substitutable, Substitution, WHNF, WHNFResult, Zonk, ZonkError,
    ctx::LevelCtx,
    rename::{Rename, RenameCtx},
};

use super::{Exp, MetaVar};

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

impl HasType for TypeUniv {
    fn typ(&self) -> Option<Box<Exp>> {
        Some(Box::new(TypeUniv::new().into()))
    }
}

impl Substitutable for TypeUniv {
    type Target = TypeUniv;

    fn subst<S: Substitution>(&self, _ctx: &mut LevelCtx, _by: &S) -> Result<Self::Target, S::Err> {
        let TypeUniv { span } = self;
        Ok(TypeUniv { span: *span })
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

impl Rename for TypeUniv {
    fn rename_in_ctx(&mut self, _ctx: &mut RenameCtx) {}
}

impl FreeVars for TypeUniv {
    fn free_vars_mut(
        &self,
        _ctx: &LevelCtx,
        _cutoff: usize,
        _fvs: &mut crate::HashSet<crate::Lvl>,
    ) {
    }
}

impl Inline for TypeUniv {
    fn inline(&mut self, _ctx: &super::Closure, _recursive: bool) {}
}

impl WHNF for TypeUniv {
    type Target = Exp;
    fn whnf(&self, _ctx: LevelCtx) -> WHNFResult<MachineState<Self::Target>> {
        Ok((self.clone().into(), IsWHNF::WHNF))
    }
}
