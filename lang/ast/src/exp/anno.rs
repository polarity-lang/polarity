use derivative::Derivative;
use miette_util::codespan::Span;
use printer::{Alloc, Builder, Precedence, Print, PrintCfg, tokens::COLON};

use crate::{
    ContainsMetaVars, HasSpan, HasType, Occurs, Shift, ShiftRange, Substitutable, Substitution,
    Zonk, ZonkError,
    ctx::LevelCtx,
    rename::{Rename, RenameCtx},
};

use super::{Exp, MetaVar};

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
    fn occurs<F>(&self, ctx: &mut LevelCtx, f: &F) -> bool
    where
        F: Fn(&LevelCtx, &Exp) -> bool,
    {
        let Anno { exp, typ, .. } = self;
        exp.occurs(ctx, f) || typ.occurs(ctx, f)
    }
}

impl HasType for Anno {
    fn typ(&self) -> Option<Box<Exp>> {
        self.normalized_type.clone()
    }
}

impl Substitutable for Anno {
    type Target = Anno;

    fn subst<S: Substitution>(&self, ctx: &mut LevelCtx, by: &S) -> Result<Self::Target, S::Err> {
        let Anno { span, exp, typ, .. } = self;
        Ok(Anno {
            span: *span,
            exp: exp.subst(ctx, by)?,
            typ: typ.subst(ctx, by)?,
            normalized_type: None,
        })
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
        exp.print(cfg, alloc).append(COLON).append(typ.print(cfg, alloc)).parens()
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

impl Rename for Anno {
    fn rename_in_ctx(&mut self, ctx: &mut RenameCtx) {
        self.exp.rename_in_ctx(ctx);
        self.typ.rename_in_ctx(ctx);
        self.normalized_type.rename_in_ctx(ctx);
    }
}
