use derivative::Derivative;
use miette_util::codespan::Span;
use pretty::DocAllocator;
use printer::{
    Alloc, Builder, Precedence, Print, PrintCfg,
    theme::ThemeExt,
    tokens::{DOT, MATCH},
};

use crate::{
    Closure, ContainsMetaVars, FreeVars, HasSpan, HasType, Occurs, Shift, ShiftRange, Subst,
    Substitutable, Zonk, ZonkError,
    ctx::{LevelCtx, values::TypeCtx},
    rename::{Rename, RenameCtx},
};

use super::{Case, Exp, Label, MetaVar, Motive, TypCtor, print_cases};

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct LocalMatch {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub span: Option<Span>,
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub ctx: Option<TypeCtx>,
    pub name: Label,
    pub closure: Closure,
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
    fn occurs<F>(&self, ctx: &mut LevelCtx, f: &F) -> bool
    where
        F: Fn(&LevelCtx, &Exp) -> bool,
    {
        let LocalMatch { on_exp, motive, ret_typ, cases, .. } = self;
        on_exp.occurs(ctx, f)
            || motive.as_ref().is_some_and(|m| m.occurs(ctx, f))
            || ret_typ.as_ref().is_some_and(|t| t.occurs(ctx, f))
            || cases.iter().any(|case| case.occurs(ctx, f))
    }
}

impl HasType for LocalMatch {
    fn typ(&self) -> Option<Box<Exp>> {
        self.inferred_type.clone().map(|x| Box::new(x.into()))
    }
}

impl Substitutable for LocalMatch {
    type Target = LocalMatch;
    fn subst(&self, ctx: &LevelCtx, subst: &Subst) -> Self::Target {
        let LocalMatch { span, name, on_exp, motive, ret_typ, cases, .. } = self;
        LocalMatch {
            span: *span,
            ctx: None,
            name: name.clone(),
            closure: self.closure.subst(ctx, subst),
            on_exp: on_exp.subst(ctx, subst),
            motive: motive.as_ref().map(|m| m.subst(ctx, subst)),
            ret_typ: ret_typ.as_ref().map(|t| t.subst(ctx, subst)),
            cases: cases.iter().map(|case| case.subst(ctx, subst)).collect::<Vec<_>>(),
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
            .print_prec(cfg, alloc, Precedence::Ops)
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
        let LocalMatch {
            span: _,
            ctx: _,
            name: _,
            closure,
            on_exp,
            motive,
            ret_typ,
            cases,
            inferred_type,
        } = self;
        on_exp.zonk(meta_vars)?;
        motive.zonk(meta_vars)?;
        ret_typ.zonk(meta_vars)?;
        inferred_type.zonk(meta_vars)?;
        closure.zonk(meta_vars)?;
        for case in cases {
            case.zonk(meta_vars)?;
        }
        Ok(())
    }
}

impl ContainsMetaVars for LocalMatch {
    fn contains_metavars(&self) -> bool {
        let LocalMatch {
            span: _,
            ctx: _,
            name: _,
            closure,
            on_exp,
            motive,
            ret_typ,
            cases,
            inferred_type,
        } = self;

        on_exp.contains_metavars()
            || closure.contains_metavars()
            || motive.contains_metavars()
            || ret_typ.contains_metavars()
            || cases.contains_metavars()
            || inferred_type.contains_metavars()
    }
}

impl Rename for LocalMatch {
    fn rename_in_ctx(&mut self, ctx: &mut RenameCtx) {
        self.ctx = None;
        self.inferred_type = None;
        self.on_exp.rename_in_ctx(ctx);
        self.motive.rename_in_ctx(ctx);
        self.ret_typ.rename_in_ctx(ctx);
        self.cases.rename_in_ctx(ctx);
    }
}

impl FreeVars for LocalMatch {
    fn free_vars_mut(&self, ctx: &LevelCtx, cutoff: usize, fvs: &mut crate::HashSet<crate::Lvl>) {
        let LocalMatch {
            span: _,
            ctx: _,
            name: _,
            closure,
            on_exp,
            motive,
            ret_typ: _,
            cases,
            inferred_type: _,
        } = self;

        on_exp.free_vars_mut(ctx, cutoff, fvs);
        closure.free_vars_mut(ctx, cutoff, fvs);
        motive.free_vars_mut(ctx, cutoff, fvs);
        cases.free_vars_mut(ctx, cutoff, fvs);
    }
}
