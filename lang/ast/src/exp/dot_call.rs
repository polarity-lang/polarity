use derivative::Derivative;
use miette_util::codespan::Span;
use pretty::DocAllocator;
use printer::{Alloc, Builder, Precedence, Print, PrintCfg, theme::ThemeExt, tokens::DOT};

use crate::{
    ContainsMetaVars, FreeVars, HasSpan, HasType, Occurs, Shift, ShiftRange, Subst, Substitutable,
    Zonk, ZonkError,
    ctx::LevelCtx,
    rename::{Rename, RenameCtx},
};

use super::{Args, Exp, IdBound, MetaVar};

/// A DotCall expression can be one of two different kinds:
/// - A destructor introduced by a codata type declaration
/// - A definition introduced at the toplevel
#[derive(Debug, Clone, Copy, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub enum DotCallKind {
    Destructor,
    Definition,
}

/// A DotCall is either a destructor or a definition, applied to a destructee
/// Examples: `e.head` `xs.append(ys)`
#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct DotCall {
    /// Source code location
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub span: Option<Span>,
    /// Whether the dotcall is a destructor or codefinition.
    pub kind: DotCallKind,
    /// The expression to which the dotcall is applied.
    /// The `e` in `e.f(e1...en)`
    pub exp: Box<Exp>,
    /// The name of the dotcall.
    /// The `f` in `e.f(e1...en)`
    pub name: IdBound,
    /// The arguments of the dotcall.
    /// The `(e1...en)` in `e.f(e1...en)`
    pub args: Args,
    /// The inferred result type of the dotcall.
    /// This type is annotated during elaboration.
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub inferred_type: Option<Box<Exp>>,
}

impl HasSpan for DotCall {
    fn span(&self) -> Option<Span> {
        self.span
    }
}

impl From<DotCall> for Exp {
    fn from(val: DotCall) -> Self {
        Exp::DotCall(val)
    }
}

impl Shift for DotCall {
    fn shift_in_range<R: ShiftRange>(&mut self, range: &R, by: (isize, isize)) {
        self.exp.shift_in_range(range, by);
        self.args.shift_in_range(range, by);
        self.inferred_type = None;
    }
}

impl Occurs for DotCall {
    fn occurs<F>(&self, ctx: &mut LevelCtx, f: &F) -> bool
    where
        F: Fn(&LevelCtx, &Exp) -> bool,
    {
        let DotCall { exp, args, .. } = self;
        exp.occurs(ctx, f) || args.occurs(ctx, f)
    }
}

impl HasType for DotCall {
    fn typ(&self) -> Option<Box<Exp>> {
        self.inferred_type.clone()
    }
}

impl Substitutable for DotCall {
    type Target = DotCall;
    fn subst(&self, ctx: &LevelCtx, subst: &Subst) -> Self::Target {
        let DotCall { span, kind, exp, name, args, .. } = self;
        DotCall {
            span: *span,
            kind: *kind,
            exp: exp.subst(ctx, subst),
            name: name.clone(),
            args: args.subst(ctx, subst),
            inferred_type: None,
        }
    }
}

impl Zonk for DotCall {
    fn zonk(
        &mut self,
        meta_vars: &crate::HashMap<MetaVar, crate::MetaVarState>,
    ) -> Result<(), ZonkError> {
        let DotCall { span: _, kind: _, exp, name: _, args, inferred_type } = self;
        exp.zonk(meta_vars)?;
        args.zonk(meta_vars)?;
        inferred_type.zonk(meta_vars)?;
        Ok(())
    }
}

impl ContainsMetaVars for DotCall {
    fn contains_metavars(&self) -> bool {
        let DotCall { span: _, kind: _, exp, name: _, args, inferred_type } = self;

        exp.contains_metavars() || args.contains_metavars() || inferred_type.contains_metavars()
    }
}

impl Print for DotCall {
    fn print_prec<'a>(
        &'a self,
        cfg: &PrintCfg,
        alloc: &'a Alloc<'a>,
        _prec: Precedence,
    ) -> Builder<'a> {
        // A series of destructors forms an aligned group
        let mut dtors_group = alloc.nil();

        // First DotCall
        dtors_group = alloc
            .text(DOT)
            .append(alloc.dtor(&self.name.id))
            .append(self.args.print(cfg, alloc))
            .append(dtors_group);

        // Remaining DotCalls
        let mut head: &Exp = &self.exp;
        while let Exp::DotCall(DotCall { exp, name, args, .. }) = &head {
            let psubst = if args.is_empty() { alloc.nil() } else { args.print(cfg, alloc) };
            dtors_group = alloc.line_().append(dtors_group);
            dtors_group =
                alloc.text(DOT).append(alloc.dtor(&name.id)).append(psubst).append(dtors_group);
            head = exp;
        }
        head.print_prec(cfg, alloc, Precedence::Ops).append(dtors_group.align().group())
    }
}

impl Rename for DotCall {
    fn rename_in_ctx(&mut self, ctx: &mut RenameCtx) {
        self.exp.rename_in_ctx(ctx);
        self.args.rename_in_ctx(ctx);
        self.inferred_type.rename_in_ctx(ctx);
    }
}

impl FreeVars for DotCall {
    fn free_vars_mut(&self, ctx: &LevelCtx, cutoff: usize, fvs: &mut crate::HashSet<crate::Lvl>) {
        let DotCall { span: _, kind: _, exp, name: _, args, inferred_type: _ } = self;
        exp.free_vars_mut(ctx, cutoff, fvs);
        args.free_vars_mut(ctx, cutoff, fvs);
    }
}
