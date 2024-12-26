use derivative::Derivative;
use miette_util::codespan::Span;
use pretty::DocAllocator;
use printer::{
    theme::ThemeExt,
    tokens::{DOT, MATCH},
    Alloc, Builder, Precedence, Print, PrintCfg,
};

use crate::{
    ctx::{values::TypeCtx, LevelCtx},
    ContainsMetaVars, HasSpan, HasType, Occurs, Shift, ShiftRange, Substitutable, Substitution,
    Zonk, ZonkError,
};

use super::{print_cases, Case, Exp, Label, Lvl, MetaVar, Motive, TypCtor};

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
