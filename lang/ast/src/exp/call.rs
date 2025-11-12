use derivative::Derivative;

use polarity_lang_miette_util::codespan::Span;
use polarity_lang_printer::{Alloc, Builder, Precedence, Print, PrintCfg, theme::ThemeExt};

use super::{Args, Exp, IdBound, MetaVar};
use crate::{
    ContainsMetaVars, FreeVars, HasSpan, HasType, Occurs, Shift, ShiftRange, Subst, Substitutable,
    Zonk, ZonkError,
    ctx::LevelCtx,
    rename::{Rename, RenameCtx},
};

/// A Call expression can be one of three different kinds:
/// - A constructor introduced by a data type declaration
/// - A codefinition introduced at the toplevel
/// - A LetBound definition introduced at the toplevel
/// - An extern declaration at the toplevel
#[derive(Debug, Clone, Copy, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub enum CallKind {
    Constructor,
    Codefinition,
    LetBound,
    Extern,
}

/// A Call invokes a constructor, a codefinition or a toplevel let-bound definition.
/// Examples: `Zero`, `Cons(True, Nil)`, `minimum(x,y)`
#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Call {
    /// Source code location
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub span: Option<Span>,
    /// Whether the call is a constructor, codefinition, let bound definition or extern.
    pub kind: CallKind,
    /// The name of the call.
    /// The `f` in `f(e1...en)`
    pub name: IdBound,
    /// The arguments to the call.
    /// The `(e1...en)` in `f(e1...en)`
    pub args: Args,
    /// The inferred result type of the call.
    /// This type is annotated during elaboration.
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub inferred_type: Option<Box<Exp>>,
}

impl HasSpan for Call {
    fn span(&self) -> Option<Span> {
        self.span
    }
}

impl From<Call> for Exp {
    fn from(val: Call) -> Self {
        Exp::Call(val)
    }
}

impl Shift for Call {
    fn shift_in_range<R: ShiftRange>(&mut self, range: &R, by: (isize, isize)) {
        self.args.shift_in_range(range, by);
        self.inferred_type = None;
    }
}

impl Occurs for Call {
    fn occurs<F>(&self, ctx: &mut LevelCtx, f: &F) -> bool
    where
        F: Fn(&LevelCtx, &Exp) -> bool,
    {
        let Call { args, .. } = self;
        args.occurs(ctx, f)
    }
}

impl HasType for Call {
    fn typ(&self) -> Option<Box<Exp>> {
        self.inferred_type.clone()
    }
}

impl Substitutable for Call {
    type Target = Call;
    fn subst(&self, ctx: &mut LevelCtx, subst: &Subst) -> Self::Target {
        let Call { span, name, args, kind, .. } = self;
        Call {
            span: *span,
            kind: *kind,
            name: name.clone(),
            args: args.subst(ctx, subst),
            inferred_type: None,
        }
    }
}

impl Print for Call {
    fn print_prec<'a>(
        &'a self,
        cfg: &PrintCfg,
        alloc: &'a Alloc<'a>,
        _prec: Precedence,
    ) -> Builder<'a> {
        let Call { name, args, .. } = self;
        alloc.ctor(&name.id).append(args.print(cfg, alloc))
    }
}

impl Zonk for Call {
    fn zonk(
        &mut self,
        meta_vars: &crate::HashMap<MetaVar, crate::MetaVarState>,
    ) -> Result<(), ZonkError> {
        let Call { span: _, kind: _, name: _, args, inferred_type } = self;
        args.zonk(meta_vars)?;
        inferred_type.zonk(meta_vars)?;
        Ok(())
    }
}

impl ContainsMetaVars for Call {
    fn contains_metavars(&self) -> bool {
        let Call { span: _, kind: _, name: _, args, inferred_type } = self;

        args.contains_metavars() || inferred_type.contains_metavars()
    }
}

impl Rename for Call {
    fn rename_in_ctx(&mut self, ctx: &mut RenameCtx) {
        self.args.rename_in_ctx(ctx);
        self.inferred_type.rename_in_ctx(ctx);
    }
}

impl FreeVars for Call {
    fn free_vars_mut(&self, ctx: &LevelCtx, cutoff: usize, fvs: &mut crate::HashSet<crate::Lvl>) {
        let Call { span: _, kind: _, name: _, args, inferred_type: _ } = self;

        args.free_vars_mut(ctx, cutoff, fvs)
    }
}
