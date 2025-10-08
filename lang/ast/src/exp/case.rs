use derivative::Derivative;
use polarity_lang_miette_util::codespan::Span;
use polarity_lang_printer::{
    Alloc, Builder, Precedence, Print, PrintCfg,
    theme::ThemeExt,
    tokens::{ABSURD, COMMA, DOT, FAT_ARROW},
    util::BracesExt,
};
use pretty::DocAllocator;

use crate::{
    ContainsMetaVars, FreeVars, Occurs, Shift, ShiftRange, ShiftRangeExt, Subst, Substitutable,
    Zonk, ZonkError,
    ctx::{BindContext, LevelCtx},
    rename::{Rename, RenameCtx},
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
    fn print_prec<'a>(
        &'a self,
        cfg: &PrintCfg,
        alloc: &'a Alloc<'a>,
        _prec: Precedence,
    ) -> Builder<'a> {
        let Pattern { span: _, is_copattern, name, params } = self;
        if *is_copattern {
            alloc.text(DOT).append(alloc.dtor(&name.id)).append(params.print(cfg, alloc))
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
    fn subst(&self, ctx: &mut LevelCtx, subst: &Subst) -> Self::Target {
        let Case { span, pattern, body } = self;
        ctx.bind_iter(pattern.params.params.iter(), |ctx| Case {
            span: *span,
            pattern: pattern.clone(),
            body: body.as_ref().map(|body| {
                let mut subst = (*subst).clone();
                subst.shift((1, 0));
                body.subst(ctx, &subst)
            }),
        })
    }
}

// Prints "{ }"
pub fn empty_braces<'a>(alloc: &'a Alloc<'a>) -> Builder<'a> {
    alloc.space().braces_anno()
}

impl Print for Case {
    fn print_prec<'a>(
        &'a self,
        cfg: &PrintCfg,
        alloc: &'a Alloc<'a>,
        _prec: Precedence,
    ) -> Builder<'a> {
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
                .append(alloc.intersperse(cases.iter().map(|x| x.print(cfg, alloc)), sep))
                .append(alloc.text(COMMA).flat_alt(alloc.nil()))
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

impl Rename for Case {
    fn rename_in_ctx(&mut self, ctx: &mut RenameCtx) {
        self.pattern.params.rename_in_ctx(ctx);

        ctx.bind_iter(self.pattern.params.params.iter(), |new_ctx| {
            self.body.rename_in_ctx(new_ctx);
        })
    }
}

impl FreeVars for Case {
    fn free_vars_mut(&self, ctx: &LevelCtx, cutoff: usize, fvs: &mut crate::HashSet<crate::Lvl>) {
        let Case { span: _, pattern: _, body } = self;

        body.free_vars_mut(ctx, cutoff + 1, fvs)
    }
}
