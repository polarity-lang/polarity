use derivative::Derivative;

use polarity_lang_miette_util::codespan::Span;
use polarity_lang_printer::{
    Precedence, Print,
    theme::ThemeExt,
    tokens::{COLON, COLONEQ, DO, LEFT_ARROW, LET, SEMICOLON},
    util::BracesExt,
};
use pretty::DocAllocator;

use super::{Exp, VarBind};
use crate::{
    ContainsMetaVars, FreeVars, HasSpan, HasType, Occurs, Shift, ShiftRangeExt, Subst,
    Substitutable, Zonk,
    ctx::{BindContext, LevelCtx},
    rename::Rename,
};

/// Do block:
/// ```text
/// do {
///   let x := foo();
///   y <- bar();
///   x
/// }
/// ```
#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct DoBlock {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub span: Span,

    /// The list of statements of a do block.
    pub statements: DoStatements,

    /// Type of the do block inferred during elaboration.
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub inferred_type: Option<Box<Exp>>,
}

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub enum DoStatements {
    Bind {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        span: Span,

        name: VarBind,
        bound: Box<Exp>,
        body: Box<DoStatements>,

        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        inferred_type: Option<Box<Exp>>,
    },
    Let {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        span: Span,

        name: VarBind,
        typ: Option<Box<Exp>>,
        bound: Box<Exp>,
        body: Box<DoStatements>,

        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        inferred_type: Option<Box<Exp>>,
    },
    Return {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        span: Span,

        exp: Box<Exp>,

        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        inferred_type: Option<Box<Exp>>,
    },
}

impl HasSpan for DoBlock {
    fn span(&self) -> Option<Span> {
        Some(self.span)
    }
}

impl Shift for DoBlock {
    fn shift_in_range<R: crate::ShiftRange>(&mut self, range: &R, by: (isize, isize)) {
        let DoBlock { span: _, statements, inferred_type } = self;
        statements.shift_in_range(range, by);
        *inferred_type = None;
    }
}

impl Shift for DoStatements {
    fn shift_in_range<R: crate::ShiftRange>(&mut self, range: &R, by: (isize, isize)) {
        match self {
            DoStatements::Bind { span: _, name: _, bound, body, inferred_type } => {
                bound.shift_in_range(range, by);
                body.shift_in_range(&range.clone().shift(1), by);
                *inferred_type = None;
            }
            DoStatements::Let { span: _, name: _, typ, bound, body, inferred_type } => {
                typ.shift_in_range(range, by);
                bound.shift_in_range(range, by);
                body.shift_in_range(&range.clone().shift(1), by);
                *inferred_type = None;
            }
            DoStatements::Return { span: _, exp, inferred_type } => {
                exp.shift_in_range(range, by);
                *inferred_type = None;
            }
        }
    }
}

impl Occurs for DoBlock {
    fn occurs<F>(&self, ctx: &mut crate::ctx::LevelCtx, f: &F) -> bool
    where
        F: Fn(&crate::ctx::LevelCtx, &Exp) -> bool,
    {
        let DoBlock { span: _, statements, inferred_type: _ } = self;
        statements.occurs(ctx, f)
    }
}

impl Occurs for DoStatements {
    fn occurs<F>(&self, ctx: &mut LevelCtx, f: &F) -> bool
    where
        F: Fn(&LevelCtx, &Exp) -> bool,
    {
        match self {
            DoStatements::Bind { span: _, name, bound, body, inferred_type: _ } => {
                bound.occurs(ctx, f) || ctx.bind_single(name.clone(), |ctx| body.occurs(ctx, f))
            }
            DoStatements::Let { span: _, name, typ, bound, body, inferred_type: _ } => {
                typ.as_ref().is_some_and(|t| t.occurs(ctx, f))
                    || bound.occurs(ctx, f)
                    || ctx.bind_single(name.clone(), |ctx| body.occurs(ctx, f))
            }
            DoStatements::Return { span: _, exp, inferred_type: _ } => exp.occurs(ctx, f),
        }
    }
}

impl HasType for DoBlock {
    fn typ(&self) -> Option<Box<Exp>> {
        self.inferred_type.clone()
    }
}

impl HasType for DoStatements {
    fn typ(&self) -> Option<Box<Exp>> {
        match self {
            DoStatements::Bind { inferred_type, .. } => inferred_type.clone(),
            DoStatements::Let { inferred_type, .. } => inferred_type.clone(),
            DoStatements::Return { inferred_type, .. } => inferred_type.clone(),
        }
    }
}

impl Substitutable for DoBlock {
    type Target = DoBlock;

    fn subst(&self, ctx: &mut LevelCtx, subst: &Subst) -> Self::Target {
        let DoBlock { span, statements, inferred_type: _ } = self;
        DoBlock { span: *span, statements: statements.subst(ctx, subst), inferred_type: None }
    }
}

impl Substitutable for DoStatements {
    type Target = DoStatements;

    fn subst(&self, ctx: &mut LevelCtx, subst: &Subst) -> Self::Target {
        match self {
            DoStatements::Bind { span, name, bound, body, inferred_type: _ } => {
                let bound = bound.subst(ctx, subst);
                ctx.bind_single(name.clone(), |ctx| {
                    let mut subst = (*subst).clone();
                    subst.shift((1, 0));
                    DoStatements::Bind {
                        span: *span,
                        name: name.clone(),
                        bound,
                        body: body.subst(ctx, &subst),
                        inferred_type: None,
                    }
                })
            }
            DoStatements::Let { span, name, typ, bound, body, inferred_type: _ } => {
                let typ = typ.subst(ctx, subst);
                let bound = bound.subst(ctx, subst);
                ctx.bind_single(name.clone(), |ctx| {
                    let mut subst = (*subst).clone();
                    subst.shift((1, 0));
                    DoStatements::Let {
                        span: *span,
                        name: name.clone(),
                        typ,
                        bound,
                        body: body.subst(ctx, &subst),
                        inferred_type: None,
                    }
                })
            }
            DoStatements::Return { span, exp, inferred_type: _ } => {
                let exp = exp.subst(ctx, subst);
                DoStatements::Return { span: *span, exp, inferred_type: None }
            }
        }
    }
}

impl Print for DoBlock {
    fn print_prec<'a>(
        &'a self,
        cfg: &polarity_lang_printer::PrintCfg,
        alloc: &'a polarity_lang_printer::Alloc<'a>,
        _prec: polarity_lang_printer::Precedence,
    ) -> polarity_lang_printer::Builder<'a> {
        let DoBlock { span: _, statements, inferred_type: _ } = self;

        let body = alloc
            .line()
            .append(statements.print(cfg, alloc))
            .nest(cfg.indent)
            .append(alloc.line())
            .braces_anno();

        alloc.keyword(DO).append(alloc.space()).append(body)
    }
}

impl Print for DoStatements {
    fn print_prec<'a>(
        &'a self,
        cfg: &polarity_lang_printer::PrintCfg,
        alloc: &'a polarity_lang_printer::Alloc<'a>,
        _prec: polarity_lang_printer::Precedence,
    ) -> polarity_lang_printer::Builder<'a> {
        match self {
            DoStatements::Bind { span: _, name, bound, body, inferred_type: _ } => {
                let head = name
                    .print(cfg, alloc)
                    .append(alloc.space())
                    .append(LEFT_ARROW)
                    .append(alloc.space())
                    .append(bound.print_prec(cfg, alloc, Precedence::NonLet))
                    .append(SEMICOLON)
                    .group();

                let body = body.print_prec(cfg, alloc, Precedence::Exp);

                head.append(alloc.hardline()).append(body)
            }
            DoStatements::Let { span: _, name, typ, bound, body, inferred_type: _ } => {
                let typ = typ.as_ref().map(|t| {
                    alloc.text(COLON).append(alloc.space()).append(t.print_prec(
                        cfg,
                        alloc,
                        polarity_lang_printer::Precedence::NonLet,
                    ))
                });

                let head = alloc
                    .keyword(LET)
                    .append(alloc.space())
                    .append(name.print(cfg, alloc))
                    .append(typ)
                    .append(alloc.space())
                    .append(COLONEQ)
                    .append(alloc.space())
                    .append(bound.print_prec(cfg, alloc, Precedence::NonLet))
                    .append(SEMICOLON)
                    .group();

                let body = body.print_prec(cfg, alloc, Precedence::Exp);

                head.append(alloc.hardline()).append(body)
            }
            DoStatements::Return { span: _, exp, inferred_type: _ } => {
                exp.print_prec(cfg, alloc, Precedence::Exp)
            }
        }
    }
}

impl Zonk for DoBlock {
    fn zonk(
        &mut self,
        meta_vars: &crate::HashMap<crate::MetaVar, crate::MetaVarState>,
    ) -> Result<(), crate::ZonkError> {
        let DoBlock { span: _, statements, inferred_type: _ } = self;
        statements.zonk(meta_vars)
    }
}

impl Zonk for DoStatements {
    fn zonk(
        &mut self,
        meta_vars: &crate::HashMap<crate::MetaVar, crate::MetaVarState>,
    ) -> Result<(), crate::ZonkError> {
        match self {
            DoStatements::Bind { span: _, name: _, bound, body, inferred_type: _ } => {
                bound.zonk(meta_vars)?;
                body.zonk(meta_vars)?;
                Ok(())
            }
            DoStatements::Let { span: _, name: _, typ, bound, body, inferred_type: _ } => {
                typ.zonk(meta_vars)?;
                bound.zonk(meta_vars)?;
                body.zonk(meta_vars)?;
                Ok(())
            }
            DoStatements::Return { span: _, exp, inferred_type: _ } => exp.zonk(meta_vars),
        }
    }
}

impl ContainsMetaVars for DoBlock {
    fn contains_metavars(&self) -> bool {
        let DoBlock { span: _, statements, inferred_type } = self;
        statements.contains_metavars() || inferred_type.contains_metavars()
    }
}

impl ContainsMetaVars for DoStatements {
    fn contains_metavars(&self) -> bool {
        match self {
            DoStatements::Bind { span: _, name: _, bound, body, inferred_type } => {
                bound.contains_metavars()
                    || body.contains_metavars()
                    || inferred_type.contains_metavars()
            }
            DoStatements::Let { span: _, name: _, typ, bound, body, inferred_type } => {
                typ.contains_metavars()
                    || bound.contains_metavars()
                    || body.contains_metavars()
                    || inferred_type.contains_metavars()
            }
            DoStatements::Return { span: _, exp, inferred_type } => {
                exp.contains_metavars() || inferred_type.contains_metavars()
            }
        }
    }
}

impl Rename for DoBlock {
    fn rename_in_ctx(&mut self, ctx: &mut crate::rename::RenameCtx) {
        let DoBlock { span: _, statements, inferred_type: _ } = self;
        statements.rename_in_ctx(ctx);
    }
}

impl Rename for DoStatements {
    fn rename_in_ctx(&mut self, ctx: &mut crate::rename::RenameCtx) {
        match self {
            DoStatements::Bind { span: _, name, bound, body, inferred_type: _ } => {
                bound.rename_in_ctx(ctx);
                ctx.bind_single(name.clone(), |ctx| {
                    body.rename_in_ctx(ctx);
                })
            }
            DoStatements::Let { span: _, name, typ, bound, body, inferred_type: _ } => {
                typ.rename_in_ctx(ctx);
                bound.rename_in_ctx(ctx);
                ctx.bind_single(name.clone(), |ctx| {
                    body.rename_in_ctx(ctx);
                })
            }
            DoStatements::Return { span: _, exp, inferred_type: _ } => exp.rename_in_ctx(ctx),
        }
    }
}

impl From<DoBlock> for Exp {
    fn from(val: DoBlock) -> Self {
        Exp::DoBlock(val)
    }
}

impl FreeVars for DoBlock {
    fn free_vars_mut(&self, ctx: &LevelCtx, cutoff: usize, fvs: &mut crate::HashSet<crate::Lvl>) {
        let DoBlock { span: _, statements, inferred_type: _ } = self;
        statements.free_vars_mut(ctx, cutoff, fvs);
    }
}

impl FreeVars for DoStatements {
    fn free_vars_mut(&self, ctx: &LevelCtx, cutoff: usize, fvs: &mut crate::HashSet<crate::Lvl>) {
        match self {
            DoStatements::Bind { span: _, name: _, bound, body, inferred_type: _ } => {
                bound.free_vars_mut(ctx, cutoff, fvs);
                body.free_vars_mut(ctx, cutoff + 1, fvs);
            }
            DoStatements::Let { span: _, name: _, typ, bound, body, inferred_type: _ } => {
                typ.free_vars_mut(ctx, cutoff, fvs);
                bound.free_vars_mut(ctx, cutoff, fvs);
                body.free_vars_mut(ctx, cutoff + 1, fvs);
            }
            DoStatements::Return { span: _, exp, inferred_type: _ } => {
                exp.free_vars_mut(ctx, cutoff, fvs)
            }
        }
    }
}
