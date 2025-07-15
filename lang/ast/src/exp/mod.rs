use std::fmt;

use derivative::Derivative;
use miette_util::codespan::Span;
use pretty::DocAllocator;
use printer::theme::ThemeExt;
use printer::tokens::{AS, FAT_ARROW};
use printer::{Alloc, Builder, Precedence, Print, PrintCfg};

use crate::ctx::{BindContext, LevelCtx};
use crate::rename::{Rename, RenameCtx};
use crate::{ContainsMetaVars, FreeVars, MachineState, WHNF, WHNFResult, Zonk, ZonkError};

use super::HasType;
use super::subst::{Substitutable, Substitution};
use super::traits::HasSpan;
use super::traits::Occurs;
use super::{Shift, ShiftRange, ShiftRangeExt, ident::*};

mod anno;
mod args;
mod call;
mod case;
mod closure;
mod dot_call;
mod hole;
mod local_comatch;
mod local_let;
mod local_match;
mod telescope_inst;
mod typ_ctor;
mod type_univ;
mod variable;

pub use anno::*;
pub use args::*;
pub use call::*;
pub use case::*;
pub use closure::*;
pub use dot_call::*;
pub use hole::*;
pub use local_comatch::*;
pub use local_let::*;
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
    LocalLet(LocalLet),
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
            Exp::LocalLet(e) => e.span(),
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
            Exp::LocalLet(e) => e.shift_in_range(range, by),
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
            Exp::LocalLet(e) => e.occurs(ctx, f),
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
            Exp::LocalLet(e) => e.typ(),
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
            Exp::LocalLet(e) => Ok(e.subst(ctx, by)?.into()),
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
            Exp::LocalLet(e) => e.print_prec(cfg, alloc, prec),
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
            Exp::LocalLet(e) => e.zonk(meta_vars),
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
            Exp::LocalLet(local_let) => local_let.contains_metavars(),
        }
    }
}

impl Rename for Exp {
    fn rename_in_ctx(&mut self, ctx: &mut RenameCtx) {
        match self {
            Exp::Variable(e) => e.rename_in_ctx(ctx),
            Exp::LocalComatch(e) => e.rename_in_ctx(ctx),
            Exp::Anno(e) => e.rename_in_ctx(ctx),
            Exp::TypCtor(e) => e.rename_in_ctx(ctx),
            Exp::Hole(e) => e.rename_in_ctx(ctx),
            Exp::TypeUniv(e) => e.rename_in_ctx(ctx),
            Exp::Call(e) => e.rename_in_ctx(ctx),
            Exp::LocalMatch(e) => e.rename_in_ctx(ctx),
            Exp::DotCall(e) => e.rename_in_ctx(ctx),
            Exp::LocalLet(e) => e.rename_in_ctx(ctx),
        }
    }
}

impl FreeVars for Exp {
    fn free_vars_mut(&self, ctx: &LevelCtx, cutoff: usize, fvs: &mut crate::HashSet<Lvl>) {
        match self {
            Exp::Variable(e) => e.free_vars_mut(ctx, cutoff, fvs),
            Exp::TypCtor(e) => e.free_vars_mut(ctx, cutoff, fvs),
            Exp::Call(e) => e.free_vars_mut(ctx, cutoff, fvs),
            Exp::DotCall(e) => e.free_vars_mut(ctx, cutoff, fvs),
            Exp::Anno(e) => e.free_vars_mut(ctx, cutoff, fvs),
            Exp::TypeUniv(e) => e.free_vars_mut(ctx, cutoff, fvs),
            Exp::LocalMatch(e) => e.free_vars_mut(ctx, cutoff, fvs),
            Exp::LocalComatch(e) => e.free_vars_mut(ctx, cutoff, fvs),
            Exp::Hole(e) => e.free_vars_mut(ctx, cutoff, fvs),
            Exp::LocalLet(e) => e.free_vars_mut(ctx, cutoff, fvs),
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
    fn print_prec<'a>(
        &'a self,
        cfg: &PrintCfg,
        alloc: &'a Alloc<'a>,
        _prec: Precedence,
    ) -> Builder<'a> {
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

impl Rename for Motive {
    fn rename_in_ctx(&mut self, ctx: &mut RenameCtx) {
        self.param.rename_in_ctx(ctx);
        ctx.bind_single(&self.param, |new_ctx| {
            self.ret_typ.rename_in_ctx(new_ctx);
        })
    }
}

impl FreeVars for Motive {
    fn free_vars_mut(&self, ctx: &LevelCtx, cutoff: usize, fvs: &mut crate::HashSet<Lvl>) {
        let Motive { span: _, param: _, ret_typ } = self;

        ret_typ.free_vars_mut(ctx, cutoff + 1, fvs)
    }
}

impl WHNF for Exp {
    type Target = Exp;
    fn whnf(&self, ctx: Closure) -> WHNFResult<MachineState<Self::Target>> {
        match self {
            Exp::Variable(variable) => variable.whnf(ctx),
            Exp::TypCtor(typ_ctor) => typ_ctor.whnf(ctx),
            Exp::Call(call) => call.whnf(ctx),
            Exp::DotCall(dot_call) => dot_call.whnf(ctx),
            Exp::Anno(anno) => anno.whnf(ctx),
            Exp::TypeUniv(type_univ) => type_univ.whnf(ctx),
            Exp::LocalMatch(local_match) => local_match.whnf(ctx),
            Exp::LocalComatch(local_comatch) => local_comatch.whnf(ctx),
            Exp::Hole(hole) => hole.whnf(ctx),
            Exp::LocalLet(local_let) => local_let.whnf(ctx),
        }
    }

    fn inline(&mut self, ctx: &Closure) {
        match self {
            Exp::Variable(variable) => variable.inline(ctx),
            Exp::TypCtor(typ_ctor) => typ_ctor.inline(ctx),
            Exp::Call(call) => call.inline(ctx),
            Exp::DotCall(dot_call) => dot_call.inline(ctx),
            Exp::Anno(anno) => anno.inline(ctx),
            Exp::TypeUniv(type_univ) => type_univ.inline(ctx),
            Exp::LocalMatch(local_match) => local_match.inline(ctx),
            Exp::LocalComatch(local_comatch) => local_comatch.inline(ctx),
            Exp::Hole(hole) => hole.inline(ctx),
            Exp::LocalLet(local_let) => local_let.inline(ctx),
        }
    }
}
