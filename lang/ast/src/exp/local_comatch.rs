use derivative::Derivative;
use miette_util::codespan::Span;
use pretty::DocAllocator;
use printer::{
    Alloc, Builder, Precedence, Print, PrintCfg,
    theme::ThemeExt,
    tokens::{ABSURD, COMATCH, FAT_ARROW},
    util::BackslashExt,
};

use crate::{
    ContainsMetaVars, HasSpan, HasType, Occurs, Shift, ShiftRange, Substitutable, Substitution,
    Zonk, ZonkError,
    ctx::{LevelCtx, values::TypeCtx},
    rename::{Rename, RenameCtx},
};

use super::{Case, Exp, Label, MetaVar, TypCtor, print_cases};

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct LocalComatch {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub span: Option<Span>,
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub ctx: Option<TypeCtx>,
    pub name: Label,
    pub is_lambda_sugar: bool,
    pub cases: Vec<Case>,
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub inferred_type: Option<TypCtor>,
}

impl HasSpan for LocalComatch {
    fn span(&self) -> Option<Span> {
        self.span
    }
}

impl From<LocalComatch> for Exp {
    fn from(val: LocalComatch) -> Self {
        Exp::LocalComatch(val)
    }
}

impl Shift for LocalComatch {
    fn shift_in_range<R: ShiftRange>(&mut self, range: &R, by: (isize, isize)) {
        self.ctx = None;
        self.cases.shift_in_range(range, by);
        self.inferred_type = None;
    }
}

impl Occurs for LocalComatch {
    fn occurs<F>(&self, ctx: &mut LevelCtx, f: &F) -> bool
    where
        F: Fn(&LevelCtx, &Exp) -> bool,
    {
        let LocalComatch { cases, .. } = self;
        cases.iter().any(|case| case.occurs(ctx, f))
    }
}

impl HasType for LocalComatch {
    fn typ(&self) -> Option<Box<Exp>> {
        self.inferred_type.clone().map(|x| Box::new(x.into()))
    }
}

impl Substitutable for LocalComatch {
    type Target = LocalComatch;

    fn subst<S: Substitution>(&self, ctx: &mut LevelCtx, by: &S) -> Result<Self::Target, S::Err> {
        let LocalComatch { span, name, is_lambda_sugar, cases, .. } = self;
        Ok(LocalComatch {
            span: *span,
            ctx: None,
            name: name.clone(),
            is_lambda_sugar: *is_lambda_sugar,
            cases: cases.iter().map(|case| case.subst(ctx, by)).collect::<Result<Vec<_>, _>>()?,
            inferred_type: None,
        })
    }
}

/// Print the Comatch as a lambda abstraction.
/// Only invoke this function if the comatch contains exactly
/// one cocase "ap" with three arguments; the function will
/// panic otherwise.
fn print_lambda_sugar<'a>(cases: &'a [Case], cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
    let Case { span: _, pattern, body } =
        cases.first().expect("Empty comatch marked as lambda sugar");

    let body = match body {
        None => alloc.keyword(ABSURD),
        Some(body) => alloc
            .text(FAT_ARROW)
            .append(alloc.line())
            .append(body.print(cfg, alloc))
            .nest(cfg.indent),
    };
    let pattern = alloc.ctor(&pattern.name.id).append(pattern.params.print(cfg, alloc));
    alloc.backslash_anno(cfg).append(pattern).append(alloc.space()).append(body).group()
}

impl Print for LocalComatch {
    fn print_prec<'a>(
        &'a self,
        cfg: &PrintCfg,
        alloc: &'a Alloc<'a>,
        _prec: Precedence,
    ) -> Builder<'a> {
        let LocalComatch { name, is_lambda_sugar, cases, .. } = self;
        if *is_lambda_sugar && cfg.print_lambda_sugar {
            print_lambda_sugar(cases, cfg, alloc)
        } else {
            alloc
                .keyword(COMATCH)
                .append(match &name.user_name {
                    Some(name) => alloc.space().append(alloc.ctor(&name.id)),
                    None => alloc.nil(),
                })
                .append(alloc.space())
                .append(print_cases(cases, cfg, alloc))
        }
    }
}

impl Zonk for LocalComatch {
    fn zonk(
        &mut self,
        meta_vars: &crate::HashMap<MetaVar, crate::MetaVarState>,
    ) -> Result<(), ZonkError> {
        let LocalComatch { span: _, ctx: _, name: _, is_lambda_sugar: _, cases, inferred_type } =
            self;
        inferred_type.zonk(meta_vars)?;
        for case in cases {
            case.zonk(meta_vars)?;
        }
        Ok(())
    }
}

impl ContainsMetaVars for LocalComatch {
    fn contains_metavars(&self) -> bool {
        let LocalComatch { span: _, ctx: _, name: _, is_lambda_sugar: _, cases, inferred_type } =
            self;

        cases.contains_metavars() || inferred_type.contains_metavars()
    }
}

impl Rename for LocalComatch {
    fn rename_in_ctx(&mut self, ctx: &mut RenameCtx) {
        self.ctx = None;
        self.inferred_type = None;
        self.cases.rename_in_ctx(ctx);
    }
}
