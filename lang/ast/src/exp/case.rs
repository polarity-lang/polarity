use derivative::Derivative;
use miette_util::codespan::Span;
use pretty::DocAllocator;
use printer::{
    theme::ThemeExt,
    tokens::{ABSURD, COMMA, DOT, FAT_ARROW},
    util::BracesExt,
    Alloc, Builder, Print, PrintCfg,
};

use crate::{
    ctx::{BindContext, LevelCtx},
    ContainsMetaVars, FreeVars, Occurs, Shift, ShiftRange, ShiftRangeExt, Substitutable,
    Substitution, Zonk, ZonkError,
};

use super::{Exp, IdBound, MetaVar, TelescopeInst};

// Pattern
//
//

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Pattern {
    pub span: Option<Span>,
    pub is_copattern: bool,
    pub name: IdBound,
    pub params: TelescopeInst,
}

impl Print for Pattern {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Pattern { span: _, is_copattern, name, params } = self;
        if *is_copattern {
            alloc.text(DOT).append(alloc.ctor(&name.id)).append(params.print(cfg, alloc))
        } else {
            alloc.ctor(&name.id).append(params.print(cfg, alloc))
        }
    }
}

impl Zonk for Pattern {
    fn zonk(
        &mut self,
        meta_vars: &crate::HashMap<MetaVar, crate::MetaVarState>,
    ) -> Result<(), ZonkError> {
        let Pattern { span: _, is_copattern: _, name: _, params } = self;
        params.zonk(meta_vars)
    }
}

// Case
//
//

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Case {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub span: Option<Span>,
    pub pattern: Pattern,
    /// Body being `None` represents an absurd pattern
    pub body: Option<Box<Exp>>,
}

impl Shift for Case {
    fn shift_in_range<R: ShiftRange>(&mut self, range: &R, by: (isize, isize)) {
        self.body.shift_in_range(&range.clone().shift(1), by);
    }
}

impl Occurs for Case {
    fn occurs<F>(&self, ctx: &mut LevelCtx, f: &F) -> bool
    where
        F: Fn(&LevelCtx, &Exp) -> bool,
    {
        let Case { pattern, body, .. } = self;
        ctx.bind_iter(pattern.params.params.iter(), |ctx| {
            body.as_ref().is_some_and(|b| b.occurs(ctx, f))
        })
    }
}

impl Substitutable for Case {
    type Target = Case;
    fn subst<S: Substitution>(&self, ctx: &mut LevelCtx, by: &S) -> Result<Self::Target, S::Err> {
        let Case { span, pattern, body } = self;
        ctx.bind_iter(pattern.params.params.iter(), |ctx| {
            Ok(Case {
                span: *span,
                pattern: pattern.clone(),
                body: body
                    .as_ref()
                    .map(|body| {
                        let mut by = (*by).clone();
                        by.shift((1, 0));
                        body.subst(ctx, &by)
                    })
                    .transpose()?,
            })
        })
    }
}

// Prints "{ }"
pub fn empty_braces<'a>(alloc: &'a Alloc<'a>) -> Builder<'a> {
    alloc.space().braces_anno()
}

impl Print for Case {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Case { span: _, pattern, body } = self;

        let body = match body {
            None => alloc.keyword(ABSURD),
            Some(body) => alloc
                .text(FAT_ARROW)
                .append(alloc.line())
                .append(body.print(cfg, alloc))
                .nest(cfg.indent),
        };

        pattern.print(cfg, alloc).append(alloc.space()).append(body).group()
    }
}

impl Zonk for Case {
    fn zonk(
        &mut self,
        meta_vars: &crate::HashMap<MetaVar, crate::MetaVarState>,
    ) -> Result<(), ZonkError> {
        let Case { span: _, pattern, body } = self;
        pattern.zonk(meta_vars)?;
        body.zonk(meta_vars)?;
        Ok(())
    }
}

pub fn print_cases<'a>(cases: &'a [Case], cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
    match cases.len() {
        0 => empty_braces(alloc),

        1 => alloc
            .line()
            .append(cases[0].print(cfg, alloc))
            .nest(cfg.indent)
            .append(alloc.line())
            .braces_anno()
            .group(),
        _ => {
            let sep = alloc.text(COMMA).append(alloc.hardline());
            alloc
                .hardline()
                .append(alloc.intersperse(cases.iter().map(|x| x.print(cfg, alloc)), sep.clone()))
                .nest(cfg.indent)
                .append(alloc.hardline())
                .braces_anno()
        }
    }
}

impl ContainsMetaVars for Case {
    fn contains_metavars(&self) -> bool {
        let Case { span: _, pattern: _, body } = self;

        body.contains_metavars()
    }
}

impl FreeVars for Case {
    fn free_vars(&self, ctx: &mut LevelCtx, cutoff: crate::Lvl) -> crate::HashSet<crate::Lvl> {
        let Case { span: _, pattern, body } = self;

        ctx.bind_iter(pattern.params.params.iter(), |ctx| body.free_vars(ctx, cutoff))
    }
}
