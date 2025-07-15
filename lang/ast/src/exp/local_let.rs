use derivative::Derivative;
use miette_util::codespan::Span;
use pretty::DocAllocator;
use printer::{
    Precedence, Print,
    theme::ThemeExt,
    tokens::{COLON, COLONEQ, LET, SEMICOLON},
};

use crate::{
    ContainsMetaVars, FreeVars, HasSpan, HasType, MachineState, Occurs, Shift, ShiftRangeExt,
    Substitutable, WHNF, WHNFResult, Zonk,
    ctx::{BindContext, LevelCtx, values::Binder},
    rename::Rename,
};

use super::{Exp, VarBind};

/// Local let bindings:
/// ```text
/// let x : t := e; e
/// let x := e ; e
/// ```
#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct LocalLet {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub span: Span,
    /// The name of the variable being bound.
    pub name: VarBind,
    /// Optionally annotated type of the bound expression.
    pub typ: Option<Box<Exp>>,
    /// Expression that is being bound to the variable.
    pub bound: Box<Exp>,
    /// The body of the let expression, which can refer to the bound variable.
    pub body: Box<Exp>,
    /// Type of the let expression inferred during elaboration.
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub inferred_type: Option<Box<Exp>>,
}

impl HasSpan for LocalLet {
    fn span(&self) -> Option<Span> {
        Some(self.span)
    }
}

impl Shift for LocalLet {
    fn shift_in_range<R: crate::ShiftRange>(&mut self, range: &R, by: (isize, isize)) {
        let LocalLet { span: _, name: _, typ, bound, body, inferred_type } = self;

        typ.shift_in_range(range, by);
        bound.shift_in_range(range, by);
        body.shift_in_range(&range.clone().shift(1), by);
        *inferred_type = None;
    }
}

impl Occurs for LocalLet {
    fn occurs<F>(&self, ctx: &mut crate::ctx::LevelCtx, f: &F) -> bool
    where
        F: Fn(&crate::ctx::LevelCtx, &Exp) -> bool,
    {
        let LocalLet { span: _, name, typ, bound, body, inferred_type: _ } = self;
        typ.as_ref().is_some_and(|t| t.occurs(ctx, f))
            || bound.occurs(ctx, f)
            || ctx.bind_single(name.clone(), |ctx| body.occurs(ctx, f))
    }
}

impl HasType for LocalLet {
    fn typ(&self) -> Option<Box<Exp>> {
        self.inferred_type.clone()
    }
}

impl Substitutable for LocalLet {
    type Target = LocalLet;

    fn subst<S: crate::Substitution>(
        &self,
        ctx: &mut crate::ctx::LevelCtx,
        by: &S,
    ) -> Result<Self::Target, S::Err> {
        let LocalLet { span, name, typ, bound, body, inferred_type: _ } = self;

        let typ = typ.subst(ctx, by)?;
        let bound = bound.subst(ctx, by)?;

        ctx.bind_single(name.clone(), |ctx| {
            Ok(LocalLet {
                span: *span,
                name: name.clone(),
                typ,
                bound,
                body: body.subst(ctx, by)?,
                inferred_type: None,
            })
        })
    }
}

impl Print for LocalLet {
    fn print_prec<'a>(
        &'a self,
        cfg: &printer::PrintCfg,
        alloc: &'a printer::Alloc<'a>,
        _prec: printer::Precedence,
    ) -> printer::Builder<'a> {
        let LocalLet { span: _, name, typ, bound, body, inferred_type: _ } = self;

        let typ = typ.as_ref().map(|t| {
            alloc.text(COLON).append(alloc.space()).append(t.print_prec(
                cfg,
                alloc,
                printer::Precedence::NonLet,
            ))
        });

        let head = alloc
            .keyword(LET)
            .append(alloc.space())
            .append(name.print(cfg, alloc))
            .append(typ)
            .append(alloc.space())
            .append(COLONEQ)
            .append(bound.print_prec(cfg, alloc, Precedence::NonLet))
            .append(SEMICOLON)
            .group();

        let body = body.print_prec(cfg, alloc, Precedence::Exp);

        head.append(alloc.hardline()).append(body)
    }
}

impl Zonk for LocalLet {
    fn zonk(
        &mut self,
        meta_vars: &crate::HashMap<crate::MetaVar, crate::MetaVarState>,
    ) -> Result<(), crate::ZonkError> {
        let LocalLet { span: _, name: _, typ, bound, body, inferred_type: _ } = self;
        typ.zonk(meta_vars)?;
        bound.zonk(meta_vars)?;
        body.zonk(meta_vars)?;
        Ok(())
    }
}

impl ContainsMetaVars for LocalLet {
    fn contains_metavars(&self) -> bool {
        let LocalLet { span: _, name: _, typ, bound, body, inferred_type } = self;
        typ.contains_metavars()
            || bound.contains_metavars()
            || body.contains_metavars()
            || inferred_type.as_ref().is_some_and(|t| t.contains_metavars())
    }
}

impl Rename for LocalLet {
    fn rename_in_ctx(&mut self, ctx: &mut crate::rename::RenameCtx) {
        let LocalLet { span: _, name, typ, bound, body, inferred_type: _ } = self;

        typ.rename_in_ctx(ctx);
        bound.rename_in_ctx(ctx);

        ctx.bind_single(name.clone(), |ctx| {
            body.rename_in_ctx(ctx);
        })
    }
}

impl From<LocalLet> for Exp {
    fn from(val: LocalLet) -> Self {
        Exp::LocalLet(val)
    }
}

impl FreeVars for LocalLet {
    fn free_vars_mut(&self, ctx: &LevelCtx, cutoff: usize, fvs: &mut crate::HashSet<crate::Lvl>) {
        let LocalLet { span: _, name: _, typ, bound, body, inferred_type: _ } = self;
        bound.free_vars_mut(ctx, cutoff, fvs);
        typ.free_vars_mut(ctx, cutoff, fvs);
        body.free_vars_mut(ctx, cutoff + 1, fvs);
    }
}

impl WHNF for LocalLet {
    type Target = Exp;

    fn whnf(&self, mut ctx: super::Closure) -> WHNFResult<MachineState<Self::Target>> {
        let LocalLet { bound, body, name, .. } = self;
        let elem: Vec<Binder<Option<Box<Exp>>>> =
            vec![Binder { name: name.clone(), content: Some(bound.clone()) }];
        ctx.args.push(elem);
        (**body).whnf(ctx)
    }

    fn inline(&mut self, ctx: &super::Closure) {
        self.bound.inline(ctx);
        self.typ.inline(ctx);
        self.body.inline(ctx);
    }
}
