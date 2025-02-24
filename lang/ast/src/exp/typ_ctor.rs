use derivative::Derivative;
use miette_util::codespan::Span;
use pretty::DocAllocator;
use printer::{theme::ThemeExt, tokens::ARROW, Alloc, Builder, Precedence, Print, PrintCfg};

use crate::{
    ctx::LevelCtx, ContainsMetaVars, HasSpan, HasType, Occurs, Shift, ShiftRange, Substitutable,
    Substitution, Zonk, ZonkError,
};

use super::{Args, Exp, IdBound, MetaVar, TypeUniv};

/// A type constructor applied to arguments. The type of `TypCtor`
/// is always the type universe `Type`.
/// Examples: `Nat`, `List(Nat)`
#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct TypCtor {
    /// Source code location
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub span: Option<Span>,
    /// Name of the type constructor
    pub name: IdBound,
    /// Arguments to the type constructor
    pub args: Args,
}

impl TypCtor {
    pub fn to_exp(&self) -> Exp {
        Exp::TypCtor(self.clone())
    }

    /// A type application is simple if the list of arguments is empty.
    pub fn is_simple(&self) -> bool {
        self.args.is_empty()
    }
}

impl HasSpan for TypCtor {
    fn span(&self) -> Option<Span> {
        self.span
    }
}

impl From<TypCtor> for Exp {
    fn from(val: TypCtor) -> Self {
        Exp::TypCtor(val)
    }
}

impl Shift for TypCtor {
    fn shift_in_range<R: ShiftRange>(&mut self, range: &R, by: (isize, isize)) {
        self.args.shift_in_range(range, by);
    }
}

impl Occurs for TypCtor {
    fn occurs<F>(&self, ctx: &mut LevelCtx, f: &F) -> bool
    where
        F: Fn(&LevelCtx, &Exp) -> bool,
    {
        let TypCtor { args, .. } = self;
        args.occurs(ctx, f)
    }
}

impl HasType for TypCtor {
    fn typ(&self) -> Option<Box<Exp>> {
        Some(Box::new(TypeUniv::new().into()))
    }
}

impl Substitutable for TypCtor {
    type Target = TypCtor;
    fn subst<S: Substitution>(&self, ctx: &mut LevelCtx, by: &S) -> Result<Self::Target, S::Err> {
        let TypCtor { span, name, args } = self;
        Ok(TypCtor { span: *span, name: name.clone(), args: args.subst(ctx, by)? })
    }
}

impl Print for TypCtor {
    fn print_prec<'a>(
        &'a self,
        cfg: &PrintCfg,
        alloc: &'a Alloc<'a>,
        prec: Precedence,
    ) -> Builder<'a> {
        let TypCtor { span: _, name, args } = self;
        if name.id == "Fun" && args.len() == 2 && cfg.print_function_sugar {
            let arg = args.args[0].print_prec(cfg, alloc, 1);
            let res = args.args[1].print_prec(cfg, alloc, 0);
            let fun = arg.append(alloc.space()).append(ARROW).append(alloc.space()).append(res);
            if prec == 0 {
                fun
            } else {
                fun.parens()
            }
        } else if cfg.html {
            return alloc.reference(name.uri.to_owned(), &name.id).append(args.print(cfg, alloc));
        } else {
            return alloc.typ(&name.id).append(args.print(cfg, alloc));
        }
    }
}

/// Implement Zonk for TypCtor
impl Zonk for TypCtor {
    fn zonk(
        &mut self,
        meta_vars: &crate::HashMap<MetaVar, crate::MetaVarState>,
    ) -> Result<(), ZonkError> {
        let TypCtor { span: _, name: _, args } = self;
        args.zonk(meta_vars)
    }
}

impl ContainsMetaVars for TypCtor {
    fn contains_metavars(&self) -> bool {
        let TypCtor { span: _, name: _, args } = self;

        args.contains_metavars()
    }
}
