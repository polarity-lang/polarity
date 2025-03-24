use std::fmt;

use derivative::Derivative;
use miette_util::codespan::Span;
use pretty::DocAllocator;
use printer::theme::ThemeExt;
use printer::tokens::{AS, FAT_ARROW};
use printer::{Alloc, Builder, Precedence, Print, PrintCfg};

use crate::ctx::{BindContext, LevelCtx};
use crate::{ContainsMetaVars, FreeVars, Zonk, ZonkError};

use super::subst::{Substitutable, Substitution};
use super::traits::HasSpan;
use super::traits::Occurs;
use super::HasType;
use super::{ident::*, Shift, ShiftRange, ShiftRangeExt};

mod anno;
mod args;
mod call;
mod case;
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
pub use case::*;
pub use dot_call::*;
pub use hole::*;
pub use local_comatch::*;
pub use local_match::*;
pub use telescope_inst::*;
pub use typ_ctor::*;
pub use type_univ::*;
pub use variable::*;

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Label {
    /// A machine-generated, unique id
    pub id: usize,
    /// A user-annotated name
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub user_name: Option<IdBind>,
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
    fn occurs<F>(&self, ctx: &mut LevelCtx, f: &F) -> bool
    where
        F: Fn(&LevelCtx, &Exp) -> bool,
    {
        if f(ctx, self) {
            return true;
        }
        match self {
            Exp::Variable(_) => {
                // Variables have no subexpressions, therefore the check above is sufficient
                false
            }
            Exp::TypCtor(e) => e.occurs(ctx, f),
            Exp::Call(e) => e.occurs(ctx, f),
            Exp::DotCall(e) => e.occurs(ctx, f),
            Exp::Anno(e) => e.occurs(ctx, f),
            Exp::TypeUniv(_) => {
                // The type universe has no subexpressions, therefore the check above is sufficient
                false
            }
            Exp::LocalMatch(e) => e.occurs(ctx, f),
            Exp::LocalComatch(e) => e.occurs(ctx, f),
            Exp::Hole(e) => e.occurs(ctx, f),
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
    type Target = Exp;
    fn subst<S: Substitution>(&self, ctx: &mut LevelCtx, by: &S) -> Result<Self::Target, S::Err> {
        match self {
            Exp::Variable(e) => Ok(*e.subst(ctx, by)?),
            Exp::TypCtor(e) => Ok(e.subst(ctx, by)?.into()),
            Exp::Call(e) => Ok(e.subst(ctx, by)?.into()),
            Exp::DotCall(e) => Ok(e.subst(ctx, by)?.into()),
            Exp::Anno(e) => Ok(e.subst(ctx, by)?.into()),
            Exp::TypeUniv(e) => Ok(e.subst(ctx, by)?.into()),
            Exp::LocalMatch(e) => Ok(e.subst(ctx, by)?.into()),
            Exp::LocalComatch(e) => Ok(e.subst(ctx, by)?.into()),
            Exp::Hole(e) => Ok(e.subst(ctx, by)?.into()),
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

impl FreeVars for Exp {
    fn free_vars(&self, ctx: &mut LevelCtx, cutoff: Lvl) -> crate::HashSet<Lvl> {
        match self {
            Exp::Variable(variable) => variable.free_vars(ctx, cutoff),
            Exp::TypCtor(typ_ctor) => typ_ctor.free_vars(ctx, cutoff),
            Exp::Call(call) => call.free_vars(ctx, cutoff),
            Exp::DotCall(dot_call) => dot_call.free_vars(ctx, cutoff),
            Exp::Anno(anno) => anno.free_vars(ctx, cutoff),
            Exp::TypeUniv(type_univ) => type_univ.free_vars(ctx, cutoff),
            Exp::LocalMatch(local_match) => local_match.free_vars(ctx, cutoff),
            Exp::LocalComatch(local_comatch) => local_comatch.free_vars(ctx, cutoff),
            Exp::Hole(hole) => hole.free_vars(ctx, cutoff),
        }
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
    type Target = Motive;
    fn subst<S: Substitution>(&self, ctx: &mut LevelCtx, by: &S) -> Result<Self::Target, S::Err> {
        let Motive { span, param, ret_typ } = self;

        Ok(Motive {
            span: *span,
            param: param.clone(),
            ret_typ: ctx.bind_single(param, |ctx| {
                let mut by = (*by).clone();
                by.shift((1, 0));
                ret_typ.subst(ctx, &by)
            })?,
        })
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

impl Occurs for Motive {
    fn occurs<F>(&self, ctx: &mut LevelCtx, f: &F) -> bool
    where
        F: Fn(&LevelCtx, &Exp) -> bool,
    {
        let Motive { param, ret_typ, .. } = self;
        ctx.bind_single(param, |ctx| ret_typ.occurs(ctx, f))
    }
}

impl FreeVars for Motive {
    fn free_vars(&self, ctx: &mut LevelCtx, cutoff: Lvl) -> crate::HashSet<Lvl> {
        let Motive { span: _, param, ret_typ } = self;
        ctx.bind_single(param, |ctx| ret_typ.free_vars(ctx, cutoff))
    }
}
