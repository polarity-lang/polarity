use std::fmt;
use std::hash::Hash;

use codespan::Span;
use derivative::Derivative;
use pretty::DocAllocator;
use printer::theme::ThemeExt;
use printer::tokens::{ABSURD, AS, COLONEQ, COMATCH, COMMA, DOT, FAT_ARROW, MATCH};
use printer::util::{BackslashExt, BracesExt};
use printer::{Alloc, Builder, Precedence, Print, PrintCfg};

use crate::ctx::values::TypeCtx;
use crate::ctx::{BindContext, LevelCtx};
use crate::named::Named;
use crate::{ContainsMetaVars, Zonk, ZonkError};

use super::subst::{Substitutable, Substitution};
use super::traits::HasSpan;
use super::traits::Occurs;
use super::HasType;
use super::{ident::*, Shift, ShiftRange, ShiftRangeExt};

mod anno;
mod call;
mod dot_call;
mod hole;
mod typ_ctor;
mod type_univ;
mod variable;
pub use anno::*;
pub use call::*;
pub use dot_call::*;
pub use hole::*;
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

// Arg
//
//

/// Arguments in an argument list can either be unnamed or named.
/// Example for named arguments: `f(x := 1, y := 2)`
/// Example for unnamed arguments: `f(1, 2)`
#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub enum Arg {
    UnnamedArg(Box<Exp>),
    NamedArg(Ident, Box<Exp>),
    InsertedImplicitArg(Hole),
}

impl Arg {
    pub fn is_inserted_implicit(&self) -> bool {
        matches!(self, Arg::InsertedImplicitArg(_))
    }

    pub fn exp(&self) -> Box<Exp> {
        match self {
            Arg::UnnamedArg(e) => e.clone(),
            Arg::NamedArg(_, e) => e.clone(),
            Arg::InsertedImplicitArg(hole) => Box::new(hole.clone().into()),
        }
    }
}

impl HasSpan for Arg {
    fn span(&self) -> Option<Span> {
        match self {
            Arg::UnnamedArg(e) => e.span(),
            Arg::NamedArg(_, e) => e.span(),
            Arg::InsertedImplicitArg(hole) => hole.span(),
        }
    }
}

impl Shift for Arg {
    fn shift_in_range<R: ShiftRange>(&mut self, range: &R, by: (isize, isize)) {
        match self {
            Arg::UnnamedArg(e) => e.shift_in_range(range, by),
            Arg::NamedArg(_, e) => e.shift_in_range(range, by),
            Arg::InsertedImplicitArg(hole) => hole.shift_in_range(range, by),
        }
    }
}

impl Occurs for Arg {
    fn occurs(&self, ctx: &mut LevelCtx, lvl: Lvl) -> bool {
        match self {
            Arg::UnnamedArg(e) => e.occurs(ctx, lvl),
            Arg::NamedArg(_, e) => e.occurs(ctx, lvl),
            Arg::InsertedImplicitArg(hole) => hole.occurs(ctx, lvl),
        }
    }
}

impl HasType for Arg {
    fn typ(&self) -> Option<Box<Exp>> {
        match self {
            Arg::UnnamedArg(e) => e.typ(),
            Arg::NamedArg(_, e) => e.typ(),
            Arg::InsertedImplicitArg(hole) => hole.typ(),
        }
    }
}

impl Substitutable for Arg {
    type Result = Arg;
    fn subst<S: Substitution>(&self, ctx: &mut LevelCtx, by: &S) -> Self::Result {
        match self {
            Arg::UnnamedArg(e) => Arg::UnnamedArg(e.subst(ctx, by)),
            Arg::NamedArg(i, e) => Arg::NamedArg(i.clone(), e.subst(ctx, by)),
            Arg::InsertedImplicitArg(hole) => Arg::InsertedImplicitArg(hole.subst(ctx, by)),
        }
    }
}

impl Print for Arg {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        match self {
            Arg::UnnamedArg(e) => e.print(cfg, alloc),
            Arg::NamedArg(i, e) => alloc.text(&i.id).append(COLONEQ).append(e.print(cfg, alloc)),
            Arg::InsertedImplicitArg(_) => {
                panic!("Inserted implicit arguments should not be printed")
            }
        }
    }
}

impl Zonk for Arg {
    fn zonk(
        &mut self,
        meta_vars: &crate::HashMap<MetaVar, crate::MetaVarState>,
    ) -> Result<(), ZonkError> {
        match self {
            Arg::UnnamedArg(e) => e.zonk(meta_vars),
            Arg::NamedArg(_, e) => e.zonk(meta_vars),
            Arg::InsertedImplicitArg(hole) => hole.zonk(meta_vars),
        }
    }
}

impl ContainsMetaVars for Arg {
    fn contains_metavars(&self) -> bool {
        match self {
            Arg::UnnamedArg(e) => e.contains_metavars(),
            Arg::NamedArg(_, e) => e.contains_metavars(),
            Arg::InsertedImplicitArg(hole) => hole.contains_metavars(),
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

// LocalMatch
//
//

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct LocalMatch {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub span: Option<Span>,
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub ctx: Option<TypeCtx>,
    pub name: Label,
    pub on_exp: Box<Exp>,
    pub motive: Option<Motive>,
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub ret_typ: Option<Box<Exp>>,
    pub cases: Vec<Case>,
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub inferred_type: Option<TypCtor>,
}

impl HasSpan for LocalMatch {
    fn span(&self) -> Option<Span> {
        self.span
    }
}

impl From<LocalMatch> for Exp {
    fn from(val: LocalMatch) -> Self {
        Exp::LocalMatch(val)
    }
}

impl Shift for LocalMatch {
    fn shift_in_range<R: ShiftRange>(&mut self, range: &R, by: (isize, isize)) {
        self.ctx = None;
        self.on_exp.shift_in_range(range, by);
        self.motive.shift_in_range(range, by);
        self.ret_typ = None;
        self.cases.shift_in_range(range, by);
        self.inferred_type = None;
    }
}

impl Occurs for LocalMatch {
    fn occurs(&self, ctx: &mut LevelCtx, lvl: Lvl) -> bool {
        let LocalMatch { on_exp, cases, .. } = self;
        on_exp.occurs(ctx, lvl) || cases.occurs(ctx, lvl)
    }
}

impl HasType for LocalMatch {
    fn typ(&self) -> Option<Box<Exp>> {
        self.inferred_type.clone().map(|x| Box::new(x.into()))
    }
}

impl Substitutable for LocalMatch {
    type Result = LocalMatch;
    fn subst<S: Substitution>(&self, ctx: &mut LevelCtx, by: &S) -> Self::Result {
        let LocalMatch { span, name, on_exp, motive, ret_typ, cases, .. } = self;
        LocalMatch {
            span: *span,
            ctx: None,
            name: name.clone(),
            on_exp: on_exp.subst(ctx, by),
            motive: motive.subst(ctx, by),
            ret_typ: ret_typ.subst(ctx, by),
            cases: cases.iter().map(|case| case.subst(ctx, by)).collect(),
            inferred_type: None,
        }
    }
}

impl Print for LocalMatch {
    fn print_prec<'a>(
        &'a self,
        cfg: &PrintCfg,
        alloc: &'a Alloc<'a>,
        _prec: Precedence,
    ) -> Builder<'a> {
        let LocalMatch { name, on_exp, motive, cases, .. } = self;
        on_exp
            .print(cfg, alloc)
            .append(DOT)
            .append(alloc.keyword(MATCH))
            .append(match &name.user_name {
                Some(name) => alloc.space().append(alloc.dtor(&name.id)),
                None => alloc.nil(),
            })
            .append(motive.as_ref().map(|m| m.print(cfg, alloc)).unwrap_or(alloc.nil()))
            .append(alloc.space())
            .append(print_cases(cases, cfg, alloc))
    }
}

impl Zonk for LocalMatch {
    fn zonk(
        &mut self,
        meta_vars: &crate::HashMap<MetaVar, crate::MetaVarState>,
    ) -> Result<(), ZonkError> {
        let LocalMatch { span: _, ctx: _, name: _, on_exp, motive, ret_typ, cases, inferred_type } =
            self;
        on_exp.zonk(meta_vars)?;
        motive.zonk(meta_vars)?;
        ret_typ.zonk(meta_vars)?;
        inferred_type.zonk(meta_vars)?;
        for case in cases {
            case.zonk(meta_vars)?;
        }
        Ok(())
    }
}

impl ContainsMetaVars for LocalMatch {
    fn contains_metavars(&self) -> bool {
        let LocalMatch { span: _, ctx: _, name: _, on_exp, motive, ret_typ, cases, inferred_type } =
            self;

        on_exp.contains_metavars()
            || motive.contains_metavars()
            || ret_typ.contains_metavars()
            || cases.contains_metavars()
            || inferred_type.contains_metavars()
    }
}

// LocalComatch
//
//

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
    fn occurs(&self, ctx: &mut LevelCtx, lvl: Lvl) -> bool {
        let LocalComatch { cases, .. } = self;
        cases.occurs(ctx, lvl)
    }
}

impl HasType for LocalComatch {
    fn typ(&self) -> Option<Box<Exp>> {
        self.inferred_type.clone().map(|x| Box::new(x.into()))
    }
}

impl Substitutable for LocalComatch {
    type Result = LocalComatch;

    fn subst<S: Substitution>(&self, ctx: &mut LevelCtx, by: &S) -> Self::Result {
        let LocalComatch { span, name, is_lambda_sugar, cases, .. } = self;
        LocalComatch {
            span: *span,
            ctx: None,
            name: name.clone(),
            is_lambda_sugar: *is_lambda_sugar,
            cases: cases.iter().map(|case| case.subst(ctx, by)).collect(),
            inferred_type: None,
        }
    }
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

/// Print the Comatch as a lambda abstraction.
/// Only invoke this function if the comatch contains exactly
/// one cocase "ap" with three arguments; the function will
/// panic otherwise.
fn print_lambda_sugar<'a>(cases: &'a [Case], cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
    let Case { pattern, body, .. } = cases.first().expect("Empty comatch marked as lambda sugar");
    let var_name = pattern
        .params
        .params
        .get(2) // The variable we want to print is at the third position: comatch { ap(_,_,x) => ...}
        .expect("No parameter bound in comatch marked as lambda sugar")
        .name();
    alloc
        .backslash_anno(cfg)
        .append(&var_name.id)
        .append(DOT)
        .append(alloc.space())
        .append(body.print(cfg, alloc))
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

// Telescope Inst
//
//

/// Instantiation of a previously declared telescope
#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct TelescopeInst {
    pub params: Vec<ParamInst>,
}

impl TelescopeInst {
    pub fn len(&self) -> usize {
        self.params.len()
    }

    pub fn is_empty(&self) -> bool {
        self.params.is_empty()
    }
}

impl Print for TelescopeInst {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        if self.params.is_empty() {
            alloc.nil()
        } else {
            self.params.print(cfg, alloc).parens()
        }
    }
}

impl Zonk for TelescopeInst {
    fn zonk(
        &mut self,
        meta_vars: &crate::HashMap<MetaVar, crate::MetaVarState>,
    ) -> Result<(), ZonkError> {
        let TelescopeInst { params } = self;

        for param in params {
            param.zonk(meta_vars)?;
        }
        Ok(())
    }
}

impl ContainsMetaVars for TelescopeInst {
    fn contains_metavars(&self) -> bool {
        let TelescopeInst { params } = self;

        params.contains_metavars()
    }
}

// ParamInst
//
//

/// Instantiation of a previously declared parameter
#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct ParamInst {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub span: Option<Span>,
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub info: Option<Box<Exp>>,
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub name: Ident,
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub typ: Option<Box<Exp>>,
}

impl Named for ParamInst {
    fn name(&self) -> &Ident {
        &self.name
    }
}

impl Print for ParamInst {
    fn print<'a>(&'a self, _cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let ParamInst { span: _, info: _, name, typ: _ } = self;
        alloc.text(&name.id)
    }
}

impl Zonk for ParamInst {
    fn zonk(
        &mut self,
        meta_vars: &crate::HashMap<MetaVar, crate::MetaVarState>,
    ) -> Result<(), ZonkError> {
        let ParamInst { span: _, info, name: _, typ } = self;

        info.zonk(meta_vars)?;
        typ.zonk(meta_vars)?;
        Ok(())
    }
}

impl ContainsMetaVars for ParamInst {
    fn contains_metavars(&self) -> bool {
        let ParamInst { span: _, info, name: _, typ } = self;

        info.contains_metavars() || typ.contains_metavars()
    }
}

// Args
//
//

/// A list of arguments
/// In dependent type theory, this concept is usually called a "substitution" but that name would be confusing in this implementation
/// because it conflicts with the operation of substitution (i.e. substituting a terms for a variable in another term).
/// In particular, while we often substitute argument lists for telescopes, this is not always the case.
/// Substitutions in the sense of argument lists are a special case of a more general concept of context morphisms.
/// Unifiers are another example of context morphisms and applying a unifier to an expression mean substituting various terms,
/// which are not necessarily part of a single argument list.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Args {
    pub args: Vec<Arg>,
}

impl Args {
    pub fn to_exps(&self) -> Vec<Box<Exp>> {
        self.args.iter().map(|arg| arg.exp().clone()).collect()
    }

    pub fn len(&self) -> usize {
        self.args.len()
    }

    pub fn is_empty(&self) -> bool {
        self.args.is_empty()
    }
}

impl Shift for Args {
    fn shift_in_range<R: ShiftRange>(&mut self, range: &R, by: (isize, isize)) {
        self.args.shift_in_range(range, by);
    }
}

impl Substitutable for Args {
    type Result = Args;
    fn subst<S: Substitution>(&self, ctx: &mut LevelCtx, by: &S) -> Self {
        Args { args: self.args.subst(ctx, by) }
    }
}

impl Print for Args {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let mut doc = alloc.nil();
        let mut first = true;

        for arg in &self.args {
            if !arg.is_inserted_implicit() {
                if !first {
                    doc = doc.append(COMMA).append(alloc.line());
                }
                doc = doc.append(arg.print(cfg, alloc));
                first = false;
            }
        }

        doc.align().parens().group()
    }
}

impl Zonk for Args {
    fn zonk(
        &mut self,
        meta_vars: &crate::HashMap<MetaVar, crate::MetaVarState>,
    ) -> Result<(), ZonkError> {
        let Args { args } = self;

        for arg in args {
            arg.zonk(meta_vars)?;
        }
        Ok(())
    }
}

impl ContainsMetaVars for Args {
    fn contains_metavars(&self) -> bool {
        let Args { args } = self;

        args.contains_metavars()
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
