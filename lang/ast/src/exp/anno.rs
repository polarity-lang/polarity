use codespan::Span;
use derivative::Derivative;
use printer::{tokens::COLON, Alloc, Builder, Precedence, Print, PrintCfg};

use crate::{
    ctx::LevelCtx, ContainsMetaVars, HasSpan, HasType, Occurs, Shift, ShiftRange, Substitutable,
    Substitution, Zonk, ZonkError,
};

use super::{Exp, Lvl, MetaVar};

/// Type annotated term `e : t`
#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Anno {
    /// Source code location
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub span: Option<Span>,
    /// The annotated term, i.e. `e` in `e : t`
    pub exp: Box<Exp>,
    /// The annotated type, i.e. `t` in `e : t`
    pub typ: Box<Exp>,
    /// The annotated type written by the user might not
    /// be in normal form. After elaboration we therefore
    /// annotate the normalized type.
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub normalized_type: Option<Box<Exp>>,
}

impl HasSpan for Anno {
    fn span(&self) -> Option<Span> {
        self.span
    }
}

impl From<Anno> for Exp {
    fn from(val: Anno) -> Self {
        Exp::Anno(val)
    }
}

impl Shift for Anno {
    fn shift_in_range<R: ShiftRange>(&mut self, range: &R, by: (isize, isize)) {
        self.exp.shift_in_range(range, by);
        self.typ.shift_in_range(range, by);
        self.normalized_type = None;
    }
}

impl Occurs for Anno {
    fn occurs(&self, ctx: &mut LevelCtx, lvl: Lvl) -> bool {
        let Anno { exp, typ, .. } = self;
        exp.occurs(ctx, lvl) || typ.occurs(ctx, lvl)
    }
}

impl HasType for Anno {
    fn typ(&self) -> Option<Box<Exp>> {
        self.normalized_type.clone()
    }
}

impl Substitutable for Anno {
    type Result = Anno;

    fn subst<S: Substitution>(&self, ctx: &mut LevelCtx, by: &S) -> Self::Result {
        let Anno { span, exp, typ, .. } = self;
        Anno {
            span: *span,
            exp: exp.subst(ctx, by),
            typ: typ.subst(ctx, by),
            normalized_type: None,
        }
    }
}

impl Print for Anno {
    fn print_prec<'a>(
        &'a self,
        cfg: &PrintCfg,
        alloc: &'a Alloc<'a>,
        _prec: Precedence,
    ) -> Builder<'a> {
        let Anno { exp, typ, .. } = self;
        exp.print(cfg, alloc).parens().append(COLON).append(typ.print(cfg, alloc))
    }
}

impl Zonk for Anno {
    fn zonk(
        &mut self,
        meta_vars: &crate::HashMap<MetaVar, crate::MetaVarState>,
    ) -> Result<(), ZonkError> {
        let Anno { span: _, exp, typ, normalized_type } = self;
        exp.zonk(meta_vars)?;
        typ.zonk(meta_vars)?;
        normalized_type.zonk(meta_vars)?;
        Ok(())
    }
}

impl ContainsMetaVars for Anno {
    fn contains_metavars(&self) -> bool {
        let Anno { span: _, exp, typ, normalized_type } = self;

        exp.contains_metavars() || typ.contains_metavars() || normalized_type.contains_metavars()
    }
}
