use std::fmt;

use codespan::Span;
use derivative::Derivative;
use pretty::DocAllocator;
use printer::theme::ThemeExt;
use printer::tokens::{ABSURD, AS, COMMA, DOT, FAT_ARROW};
use printer::util::BracesExt;
use printer::{Alloc, Builder, Precedence, Print, PrintCfg};

use crate::ctx::{BindContext, LevelCtx};
use crate::{ContainsMetaVars, Zonk, ZonkError};

use super::subst::{Substitutable, Substitution};
use super::traits::HasSpan;
use super::traits::Occurs;
use super::HasType;
use super::{ident::*, Shift, ShiftRange, ShiftRangeExt};

mod anno;
mod args;
mod call;
mod dot_call;
mod hole;
mod local_comatch;
mod local_match;
mod telescope_inst;
mod typ_ctor;
mod type_univ;
mod variable;
pub use anno::*;
pub use args::*;
pub use call::*;
pub use dot_call::*;
pub use hole::*;
pub use local_comatch::*;
pub use local_match::*;
pub use telescope_inst::*;
pub use typ_ctor::*;
pub use type_univ::*;
pub use variable::*;

// Prints "{ }"
pub fn empty_braces<'a>(alloc: &'a Alloc<'a>) -> Builder<'a> {
    alloc.space().braces_anno()
}

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Label {
    /// A machine-generated, unique id
    pub id: usize,
    /// A user-annotated name
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub user_name: Option<Ident>,
}

impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.user_name {
            None => Ok(()),
            Some(user_name) => user_name.fmt(f),
        }
    }
}

// Exp
//
//

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub enum Exp {
    Variable(Variable),
    TypCtor(TypCtor),
    Call(Call),
    DotCall(DotCall),
    Anno(Anno),
    TypeUniv(TypeUniv),
    LocalMatch(LocalMatch),
    LocalComatch(LocalComatch),
    Hole(Hole),
}

impl Exp {
    pub fn to_typctor(self) -> Option<TypCtor> {
        match self {
            Exp::TypCtor(e) => Some(e),
            _ => None,
        }
    }
}

impl HasSpan for Exp {
    fn span(&self) -> Option<Span> {
        match self {
            Exp::Variable(e) => e.span(),
            Exp::TypCtor(e) => e.span(),
            Exp::Call(e) => e.span(),
            Exp::DotCall(e) => e.span(),
            Exp::Anno(e) => e.span(),
            Exp::TypeUniv(e) => e.span(),
            Exp::LocalMatch(e) => e.span(),
            Exp::LocalComatch(e) => e.span(),
            Exp::Hole(e) => e.span(),
        }
    }
}

impl Shift for Exp {
    fn shift_in_range<R: ShiftRange>(&mut self, range: &R, by: (isize, isize)) {
        match self {
            Exp::Variable(e) => e.shift_in_range(range, by),
            Exp::TypCtor(e) => e.shift_in_range(range, by),
            Exp::Call(e) => e.shift_in_range(range, by),
            Exp::DotCall(e) => e.shift_in_range(range, by),
            Exp::Anno(e) => e.shift_in_range(range, by),
            Exp::TypeUniv(e) => e.shift_in_range(range, by),
            Exp::LocalMatch(e) => e.shift_in_range(range, by),
            Exp::LocalComatch(e) => e.shift_in_range(range, by),
            Exp::Hole(e) => e.shift_in_range(range, by),
        }
    }
}

impl Occurs for Exp {
    fn occurs(&self, ctx: &mut LevelCtx, lvl: Lvl) -> bool {
        match self {
            Exp::Variable(e) => e.occurs(ctx, lvl),
            Exp::TypCtor(e) => e.occurs(ctx, lvl),
            Exp::Call(e) => e.occurs(ctx, lvl),
            Exp::DotCall(e) => e.occurs(ctx, lvl),
            Exp::Anno(e) => e.occurs(ctx, lvl),
            Exp::TypeUniv(e) => e.occurs(ctx, lvl),
            Exp::LocalMatch(e) => e.occurs(ctx, lvl),
            Exp::LocalComatch(e) => e.occurs(ctx, lvl),
            Exp::Hole(e) => e.occurs(ctx, lvl),
        }
    }
}

impl HasType for Exp {
    fn typ(&self) -> Option<Box<Exp>> {
        match self {
            Exp::Variable(e) => e.typ(),
            Exp::TypCtor(e) => e.typ(),
            Exp::Call(e) => e.typ(),
            Exp::DotCall(e) => e.typ(),
            Exp::Anno(e) => e.typ(),
            Exp::TypeUniv(e) => e.typ(),
            Exp::LocalMatch(e) => e.typ(),
            Exp::LocalComatch(e) => e.typ(),
            Exp::Hole(e) => e.typ(),
        }
    }
}

impl Substitutable for Exp {
    type Result = Exp;
    fn subst<S: Substitution>(&self, ctx: &mut LevelCtx, by: &S) -> Self::Result {
        match self {
            Exp::Variable(e) => *e.subst(ctx, by),
            Exp::TypCtor(e) => e.subst(ctx, by).into(),
            Exp::Call(e) => e.subst(ctx, by).into(),
            Exp::DotCall(e) => e.subst(ctx, by).into(),
            Exp::Anno(e) => e.subst(ctx, by).into(),
            Exp::TypeUniv(e) => e.subst(ctx, by).into(),
            Exp::LocalMatch(e) => e.subst(ctx, by).into(),
            Exp::LocalComatch(e) => e.subst(ctx, by).into(),
            Exp::Hole(e) => e.subst(ctx, by).into(),
        }
    }
}

impl Print for Exp {
    fn print_prec<'a>(
        &'a self,
        cfg: &PrintCfg,
        alloc: &'a Alloc<'a>,
        prec: Precedence,
    ) -> Builder<'a> {
        match self {
            Exp::Variable(e) => e.print_prec(cfg, alloc, prec),
            Exp::TypCtor(e) => e.print_prec(cfg, alloc, prec),
            Exp::Call(e) => e.print_prec(cfg, alloc, prec),
            Exp::DotCall(e) => e.print_prec(cfg, alloc, prec),
            Exp::Anno(e) => e.print_prec(cfg, alloc, prec),
            Exp::TypeUniv(e) => e.print_prec(cfg, alloc, prec),
            Exp::LocalMatch(e) => e.print_prec(cfg, alloc, prec),
            Exp::LocalComatch(e) => e.print_prec(cfg, alloc, prec),
            Exp::Hole(e) => e.print_prec(cfg, alloc, prec),
        }
    }
}

impl Zonk for Exp {
    fn zonk(
        &mut self,
        meta_vars: &crate::HashMap<MetaVar, crate::MetaVarState>,
    ) -> Result<(), ZonkError> {
        match self {
            Exp::Variable(e) => e.zonk(meta_vars),
            Exp::TypCtor(e) => e.zonk(meta_vars),
            Exp::Call(e) => e.zonk(meta_vars),
            Exp::DotCall(e) => e.zonk(meta_vars),
            Exp::Anno(e) => e.zonk(meta_vars),
            Exp::TypeUniv(e) => e.zonk(meta_vars),
            Exp::LocalMatch(e) => e.zonk(meta_vars),
            Exp::LocalComatch(e) => e.zonk(meta_vars),
            Exp::Hole(e) => e.zonk(meta_vars),
        }
    }
}

impl ContainsMetaVars for Exp {
    fn contains_metavars(&self) -> bool {
        match self {
            Exp::Variable(variable) => variable.contains_metavars(),
            Exp::TypCtor(typ_ctor) => typ_ctor.contains_metavars(),
            Exp::Call(call) => call.contains_metavars(),
            Exp::DotCall(dot_call) => dot_call.contains_metavars(),
            Exp::Anno(anno) => anno.contains_metavars(),
            Exp::TypeUniv(type_univ) => type_univ.contains_metavars(),
            Exp::LocalMatch(local_match) => local_match.contains_metavars(),
            Exp::LocalComatch(local_comatch) => local_comatch.contains_metavars(),
            Exp::Hole(hole) => hole.contains_metavars(),
        }
    }
}

// Pattern
//
//

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Pattern {
    pub is_copattern: bool,
    pub name: Ident,
    pub params: TelescopeInst,
}

impl Print for Pattern {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Pattern { is_copattern, name, params } = self;
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
        let Pattern { is_copattern: _, name: _, params } = self;
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
    fn occurs(&self, ctx: &mut LevelCtx, lvl: Lvl) -> bool {
        let Case { pattern, body, .. } = self;
        ctx.bind_iter(pattern.params.params.iter().map(|_| ()), |ctx| body.occurs(ctx, lvl))
    }
}

impl Substitutable for Case {
    type Result = Case;
    fn subst<S: Substitution>(&self, ctx: &mut LevelCtx, by: &S) -> Self {
        let Case { span, pattern, body } = self;
        ctx.bind_iter(pattern.params.params.iter(), |ctx| Case {
            span: *span,
            pattern: pattern.clone(),
            body: body.as_ref().map(|body| {
                let mut by = (*by).clone();
                by.shift((1, 0));
                body.subst(ctx, &by)
            }),
        })
    }
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

// Motive
//
//

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Motive {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub span: Option<Span>,
    pub param: ParamInst,
    pub ret_typ: Box<Exp>,
}

impl Shift for Motive {
    fn shift_in_range<R: ShiftRange>(&mut self, range: &R, by: (isize, isize)) {
        self.ret_typ.shift_in_range(&range.clone().shift(1), by);
    }
}

impl Substitutable for Motive {
    type Result = Motive;
    fn subst<S: Substitution>(&self, ctx: &mut LevelCtx, by: &S) -> Self::Result {
        let Motive { span, param, ret_typ } = self;

        Motive {
            span: *span,
            param: param.clone(),
            ret_typ: ctx.bind_single((), |ctx| {
                let mut by = (*by).clone();
                by.shift((1, 0));
                ret_typ.subst(ctx, &by)
            }),
        }
    }
}

impl Print for Motive {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Motive { span: _, param, ret_typ } = self;

        alloc
            .space()
            .append(alloc.keyword(AS))
            .append(alloc.space())
            .append(param.print(cfg, alloc))
            .append(alloc.space())
            .append(alloc.text(FAT_ARROW))
            .append(alloc.space())
            .append(ret_typ.print(cfg, alloc))
    }
}

impl Zonk for Motive {
    fn zonk(
        &mut self,
        meta_vars: &crate::HashMap<MetaVar, crate::MetaVarState>,
    ) -> Result<(), ZonkError> {
        let Motive { span: _, param, ret_typ } = self;
        param.zonk(meta_vars)?;
        ret_typ.zonk(meta_vars)?;
        Ok(())
    }
}

impl ContainsMetaVars for Motive {
    fn contains_metavars(&self) -> bool {
        let Motive { span: _, param, ret_typ } = self;

        param.contains_metavars() || ret_typ.contains_metavars()
    }
}
