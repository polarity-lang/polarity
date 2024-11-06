use codespan::Span;
use derivative::Derivative;
use printer::{theme::ThemeExt, Alloc, Builder, Precedence, Print, PrintCfg};

use crate::{
    ctx::LevelCtx, ContainsMetaVars, HasSpan, HasType, Occurs, Shift, ShiftRange, Substitutable,
    Substitution, Zonk, ZonkError,
};

use super::{Args, Exp, Ident, Lvl, MetaVar};

/// A Call expression can be one of three different kinds:
/// - A constructor introduced by a data type declaration
/// - A codefinition introduced at the toplevel
/// - A LetBound definition introduced at the toplevel
#[derive(Debug, Clone, Copy, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub enum CallKind {
    Constructor,
    Codefinition,
    LetBound,
}

/// A Call invokes a constructor, a codefinition or a toplevel let-bound definition.
/// Examples: `Zero`, `Cons(True, Nil)`, `minimum(x,y)`
#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Call {
    /// Source code location
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub span: Option<Span>,
    /// Whether the call is a constructor, codefinition or let bound definition.
    pub kind: CallKind,
    /// The name of the call.
    /// The `f` in `f(e1...en)`
    pub name: Ident,
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
    fn occurs(&self, ctx: &mut LevelCtx, lvl: Lvl) -> bool {
        let Call { args, .. } = self;
        args.args.iter().any(|arg| arg.occurs(ctx, lvl))
    }
}

impl HasType for Call {
    fn typ(&self) -> Option<Box<Exp>> {
        self.inferred_type.clone()
    }
}

impl Substitutable for Call {
    type Result = Call;
    fn subst<S: Substitution>(&self, ctx: &mut LevelCtx, by: &S) -> Self::Result {
        let Call { span, name, args, kind, .. } = self;
        Call {
            span: *span,
            kind: *kind,
            name: name.clone(),
            args: args.subst(ctx, by),
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
