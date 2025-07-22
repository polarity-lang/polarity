use derivative::Derivative;
use miette_util::codespan::Span;
use pretty::DocAllocator;
use printer::{
    Alloc, Builder, Precedence, Print, PrintCfg,
    theme::ThemeExt,
    tokens::{ABSURD, COMMA, DOT, FAT_ARROW},
    util::BracesExt,
};

use crate::{
    Closure, ContainsMetaVars, FreeVars, Inline, Occurs, Shift, ShiftRange, ShiftRangeExt,
    Substitutable, Substitution, Zonk, ZonkError,
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

#[cfg(test)]
impl Case {
    pub fn mk_test_cocase(name: &str, num_args: u64, exp: Exp) -> Case {
        use crate::ParamInst;
        use url::Url;

        use crate::VarBind;

        let mut params: Vec<ParamInst> = Vec::new();
        for i in 0..num_args {
            params.push(ParamInst {
                span: None,
                name: VarBind::Var { span: None, id: format!("x{i}") },
                typ: None,
                erased: false,
            });
        }
        Case {
            span: None,
            pattern: Pattern {
                span: None,
                is_copattern: true,
                name: IdBound {
                    span: None,
                    id: name.to_string(),
                    uri: Url::parse("inmemory://scratch.pol").unwrap(),
                },
                params: TelescopeInst { params },
            },
            body: Some(Box::new(exp)),
        }
    }
}
impl Inline for Case {
    fn inline(&mut self, ctx: &Closure, recursive: bool) {
        self.body.inline(ctx, recursive);
    }
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
