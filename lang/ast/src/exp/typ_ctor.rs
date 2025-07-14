use derivative::Derivative;
use miette_util::codespan::Span;
use pretty::DocAllocator;
use printer::{Alloc, Builder, Precedence, Print, PrintCfg, theme::ThemeExt, util::ParensIfExt};

use crate::{
    ContainsMetaVars, FreeVars, HasSpan, HasType, Occurs, Shift, ShiftRange, Substitutable,
    Substitution, Zonk, ZonkError,
    ctx::LevelCtx,
    rename::{Rename, RenameCtx},
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
    /// If this TypCtor has been lowered from a binary operator.
    ///
    /// If the user has written "->" then we populate this field with `Some("->")`
    pub is_bin_op: Option<String>,
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
        let TypCtor { span, name, args, is_bin_op } = self;
        Ok(TypCtor {
            span: *span,
            name: name.clone(),
            args: args.subst(ctx, by)?,
            is_bin_op: is_bin_op.clone(),
        })
    }
}

impl Print for TypCtor {
    fn print_prec<'a>(
        &'a self,
        cfg: &PrintCfg,
        alloc: &'a Alloc<'a>,
        prec: Precedence,
    ) -> Builder<'a> {
        let TypCtor { span: _, name, args, is_bin_op } = self;
        match is_bin_op {
            Some(op) if cfg.print_function_sugar => {
                assert!(args.len() == 2);
                // TODO: Currently this only works for right associative operators at the same precedence level (e.g. ->).
                // We still need to properly implement precedence for user-defined operators.
                let arg = args.args[0].print_prec(cfg, alloc, Precedence::Ops);
                let res = args.args[1].print_prec(cfg, alloc, Precedence::Exp);
                let fun = arg.append(alloc.space()).append(op).append(alloc.space()).append(res);
                fun.parens_if(prec > Precedence::NonLet)
            }
            _ => alloc
                .typ(&name.id)
                .annotate(printer::Anno::Reference {
                    module_uri: name.uri.to_owned(),
                    name: name.id.clone(),
                })
                .append(args.print(cfg, alloc)),
        }
    }
}

/// Implement Zonk for TypCtor
impl Zonk for TypCtor {
    fn zonk(
        &mut self,
        meta_vars: &crate::HashMap<MetaVar, crate::MetaVarState>,
    ) -> Result<(), ZonkError> {
        let TypCtor { span: _, name: _, args, is_bin_op: _ } = self;
        args.zonk(meta_vars)
    }
}

impl ContainsMetaVars for TypCtor {
    fn contains_metavars(&self) -> bool {
        let TypCtor { span: _, name: _, args, is_bin_op: _ } = self;

        args.contains_metavars()
    }
}

impl Rename for TypCtor {
    fn rename_in_ctx(&mut self, ctx: &mut RenameCtx) {
        self.args.rename_in_ctx(ctx);
    }
}

impl FreeVars for TypCtor {
    fn free_vars(&self, ctx: &LevelCtx, cutoff: usize) -> crate::HashSet<crate::Lvl> {
        let TypCtor { span: _, name: _, args, is_bin_op: _ } = self;

        args.free_vars(ctx, cutoff)
    }
}
