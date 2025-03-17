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
    Zonk, ZonkError, IdBound
};

use super::{print_cases, Case, Exp, Label, MetaVar, Motive, TypCtor};

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
    pub cases: Cases,
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
            || cases.occurs(ctx, f)
    }
}

impl HasType for LocalMatch {
    fn typ(&self) -> Option<Box<Exp>> {
        self.inferred_type.clone().map(|x| Box::new(x.into()))
    }
}

impl Substitutable for LocalMatch {
    type Target = LocalMatch;
    fn subst<S: Substitution>(&self, ctx: &mut LevelCtx, by: &S) -> Result<Self::Target, S::Err> {
        let LocalMatch { span, name, on_exp, motive, ret_typ, cases, .. } = self;
        Ok(LocalMatch {
            span: *span,
            ctx: None,
            name: name.clone(),
            on_exp: on_exp.subst(ctx, by)?,
            motive: motive.as_ref().map(|m| m.subst(ctx, by)).transpose()?,
            ret_typ: ret_typ.as_ref().map(|t| t.subst(ctx, by)).transpose()?,
            cases: cases.subst(ctx, by)?,
            inferred_type: None,
        })
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
            .append(cases.print(cfg, alloc))
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
        cases.zonk(meta_vars)?;
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


// Cases
//
//

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub enum Cases {
    Unchecked{ cases : Vec<Case> },
    Checked{
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        cases : Vec<Case>,
        args : Vec<Exp>,
        lifted_def : IdBound }
}


impl Shift for Cases {
    fn shift_in_range<R: ShiftRange>(&mut self, range: &R, by: (isize, isize)) {
        use Cases::*;
        match self {
            Unchecked { cases } => cases.shift_in_range(range, by),
            Checked { cases, args, lifted_def:_ } => {
                cases.shift_in_range(range, by);
                args.shift_in_range(range, by);
            }
        }
    }
}

impl Occurs for Cases {
    fn occurs<F>(&self, ctx: &mut LevelCtx, f: &F) -> bool
    where
        F: Fn(&LevelCtx, &Exp) -> bool,
    {
        use Cases::*;
        match self {
            Unchecked { cases } => cases.occurs(ctx, f),
            Checked { cases:_, args, lifted_def:_ } => args.occurs(ctx,f),
        }
    }
}

impl Substitutable for Cases {
    type Target = Cases;
    fn subst<S: Substitution>(&self, ctx: &mut crate::ctx::GenericCtx<()>, by: &S) -> Result<Self::Target, S::Err> {
        use Cases::*;
        match self {
            Unchecked { cases } => {
                Ok(Unchecked {
                    cases : cases.subst(ctx, by)?
                })
            },
            Checked { cases, args, lifted_def} => {
                Ok(Checked {
                    cases : cases.clone(),
                    args : args.subst(ctx,by)?,
                    lifted_def : lifted_def.clone()
                })
            },
        }
    }
}

impl Print for Cases {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        use Cases::*;
        match self {
            Unchecked { cases } => print_cases(cases, cfg, alloc),
            Checked { cases, args: _, lifted_def:_ } => print_cases(cases, cfg, alloc)
        }
    }
}

impl Zonk for Cases {
    fn zonk(&mut self, meta_vars: &crate::HashMap<MetaVar, crate::MetaVarState>) -> Result<(), ZonkError> {
        use Cases::*;
        match self {
            Unchecked { cases } => cases.zonk(meta_vars),
            Checked { cases:_, args, lifted_def:_ } => args.zonk(meta_vars),
        }
    }
}

impl ContainsMetaVars for Cases {
    fn contains_metavars(&self) -> bool {
        use Cases::*;
        match self {
            Unchecked { cases } => cases.contains_metavars(),
            Checked { cases:_, args, lifted_def:_ } => args.contains_metavars() || todo!("implement metavar check for the lifted def"),
        }
    }
}
