use derivative::Derivative;
use miette_util::codespan::Span;
use pretty::DocAllocator;
use printer::{
    Alloc, Builder, Precedence, Print, PrintCfg,
    tokens::{COLONEQ, COMMA},
};

use crate::{
    ContainsMetaVars, FreeVars, HasSpan, HasType, Occurs, Shift, ShiftRange, Subst, Substitutable,
    Substitution, SubstitutionNew, Zonk, ZonkError,
    ctx::LevelCtx,
    rename::{Rename, RenameCtx},
};

use super::{Exp, Hole, MetaVar, VarBound};

// Arg
//
//

/// Arguments in an argument list can either be unnamed or named.
/// Example for named arguments: `f(x := 1, y := 2)`
/// Example for unnamed arguments: `f(1, 2)`
#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub enum Arg {
    UnnamedArg { arg: Box<Exp>, erased: bool },
    NamedArg { name: VarBound, arg: Box<Exp>, erased: bool },
    InsertedImplicitArg { hole: Hole, erased: bool },
}

impl Arg {
    pub fn is_inserted_implicit(&self) -> bool {
        matches!(self, Arg::InsertedImplicitArg { .. })
    }

    pub fn exp(&self) -> Box<Exp> {
        match self {
            Arg::UnnamedArg { arg, .. } => arg.clone(),
            Arg::NamedArg { arg, .. } => arg.clone(),
            Arg::InsertedImplicitArg { hole, .. } => Box::new(hole.clone().into()),
        }
    }

    pub fn set_erased(&mut self, erased: bool) {
        match self {
            Arg::UnnamedArg { erased: e, .. } => *e = erased,
            Arg::NamedArg { erased: e, .. } => *e = erased,
            Arg::InsertedImplicitArg { erased: e, .. } => *e = erased,
        }
    }

    pub fn erased(&self) -> bool {
        match self {
            Arg::UnnamedArg { erased, .. } => *erased,
            Arg::NamedArg { erased, .. } => *erased,
            Arg::InsertedImplicitArg { erased, .. } => *erased,
        }
    }
}

impl HasSpan for Arg {
    fn span(&self) -> Option<Span> {
        match self {
            Arg::UnnamedArg { arg, .. } => arg.span(),
            Arg::NamedArg { arg, .. } => arg.span(),
            Arg::InsertedImplicitArg { hole, .. } => hole.span(),
        }
    }
}

impl Shift for Arg {
    fn shift_in_range<R: ShiftRange>(&mut self, range: &R, by: (isize, isize)) {
        match self {
            Arg::UnnamedArg { arg, .. } => arg.shift_in_range(range, by),
            Arg::NamedArg { arg, .. } => arg.shift_in_range(range, by),
            Arg::InsertedImplicitArg { hole, .. } => hole.shift_in_range(range, by),
        }
    }
}

impl Occurs for Arg {
    fn occurs<F>(&self, ctx: &mut LevelCtx, f: &F) -> bool
    where
        F: Fn(&LevelCtx, &Exp) -> bool,
    {
        match self {
            Arg::UnnamedArg { arg, .. } => arg.occurs(ctx, f),
            Arg::NamedArg { arg, .. } => arg.occurs(ctx, f),
            Arg::InsertedImplicitArg { hole, .. } => hole.occurs(ctx, f),
        }
    }
}

impl HasType for Arg {
    fn typ(&self) -> Option<Box<Exp>> {
        match self {
            Arg::UnnamedArg { arg, .. } => arg.typ(),
            Arg::NamedArg { arg, .. } => arg.typ(),
            Arg::InsertedImplicitArg { hole, .. } => hole.typ(),
        }
    }
}

impl Substitutable for Arg {
    type Target = Arg;
    fn subst<S: Substitution>(&self, ctx: &mut LevelCtx, by: &S) -> Result<Self::Target, S::Err> {
        match self {
            Arg::UnnamedArg { arg, erased } => {
                Ok(Arg::UnnamedArg { arg: arg.subst(ctx, by)?, erased: *erased })
            }
            Arg::NamedArg { name: var, arg, erased } => {
                Ok(Arg::NamedArg { name: var.clone(), arg: arg.subst(ctx, by)?, erased: *erased })
            }
            Arg::InsertedImplicitArg { hole, erased } => {
                Ok(Arg::InsertedImplicitArg { hole: hole.subst(ctx, by)?, erased: *erased })
            }
        }
    }
}

impl SubstitutionNew for Arg {
    type Target = Arg;
    fn subst_new(&self, ctx: &LevelCtx, subst: &Subst) -> Self::Target {
        match self {
            Arg::UnnamedArg { arg, erased } => {
                Arg::UnnamedArg { arg: arg.subst_new(ctx, subst), erased: *erased }
            }
            Arg::NamedArg { name: var, arg, erased } => {
                Arg::NamedArg { name: var.clone(), arg: arg.subst_new(ctx, subst), erased: *erased }
            }
            Arg::InsertedImplicitArg { hole, erased } => {
                Arg::InsertedImplicitArg { hole: hole.subst_new(ctx, subst), erased: *erased }
            }
        }
    }
}

impl Print for Arg {
    fn print_prec<'a>(
        &'a self,
        cfg: &PrintCfg,
        alloc: &'a Alloc<'a>,
        _prec: Precedence,
    ) -> Builder<'a> {
        match self {
            Arg::UnnamedArg { arg, .. } => arg.print(cfg, alloc),
            Arg::NamedArg { name: var, arg, .. } => {
                alloc.text(&var.id).append(COLONEQ).append(arg.print(cfg, alloc))
            }
            Arg::InsertedImplicitArg { .. } => {
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
            Arg::UnnamedArg { arg, .. } => arg.zonk(meta_vars),
            Arg::NamedArg { arg, .. } => arg.zonk(meta_vars),
            Arg::InsertedImplicitArg { hole, .. } => hole.zonk(meta_vars),
        }
    }
}

impl ContainsMetaVars for Arg {
    fn contains_metavars(&self) -> bool {
        match self {
            Arg::UnnamedArg { arg, .. } => arg.contains_metavars(),
            Arg::NamedArg { arg, .. } => arg.contains_metavars(),
            Arg::InsertedImplicitArg { hole, .. } => hole.contains_metavars(),
        }
    }
}

impl Rename for Arg {
    fn rename_in_ctx(&mut self, ctx: &mut RenameCtx) {
        match self {
            Arg::UnnamedArg { arg, .. } => arg.rename_in_ctx(ctx),
            Arg::NamedArg { arg, .. } => arg.rename_in_ctx(ctx),
            Arg::InsertedImplicitArg { hole, .. } => hole.rename_in_ctx(ctx),
        }
    }
}

impl FreeVars for Arg {
    fn free_vars_mut(&self, ctx: &LevelCtx, cutoff: usize, fvs: &mut crate::HashSet<crate::Lvl>) {
        match self {
            Arg::UnnamedArg { arg, erased: _ } => arg.free_vars_mut(ctx, cutoff, fvs),
            Arg::NamedArg { name: _, arg, erased: _ } => arg.free_vars_mut(ctx, cutoff, fvs),
            Arg::InsertedImplicitArg { hole, erased: _ } => hole.free_vars_mut(ctx, cutoff, fvs),
        }
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

impl Occurs for Args {
    fn occurs<F>(&self, ctx: &mut LevelCtx, f: &F) -> bool
    where
        F: Fn(&LevelCtx, &Exp) -> bool,
    {
        self.args.iter().any(|arg| arg.occurs(ctx, f))
    }
}

impl Substitutable for Args {
    type Target = Args;
    fn subst<S: Substitution>(&self, ctx: &mut LevelCtx, by: &S) -> Result<Self::Target, S::Err> {
        let args = self.args.iter().map(|arg| arg.subst(ctx, by)).collect::<Result<Vec<_>, _>>()?;
        Ok(Args { args })
    }
}

impl SubstitutionNew for Args {
    type Target = Args;
    fn subst_new(&self, ctx: &LevelCtx, subst: &Subst) -> Self::Target {
        let args = self.args.iter().map(|arg| arg.subst_new(ctx, subst)).collect::<Vec<_>>();
        Args { args }
    }
}
impl Print for Args {
    fn print_prec<'a>(
        &'a self,
        cfg: &PrintCfg,
        alloc: &'a Alloc<'a>,
        _prec: Precedence,
    ) -> Builder<'a> {
        if !self.args.iter().any(|x| !x.is_inserted_implicit()) {
            return alloc.nil();
        }

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

impl Rename for Args {
    fn rename_in_ctx(&mut self, ctx: &mut RenameCtx) {
        self.args.rename_in_ctx(ctx);
    }
}

impl FreeVars for Args {
    fn free_vars_mut(&self, ctx: &LevelCtx, cutoff: usize, fvs: &mut crate::HashSet<crate::Lvl>) {
        let Args { args } = self;
        for arg in args {
            arg.free_vars_mut(ctx, cutoff, fvs);
        }
    }
}

#[cfg(test)]
mod args_tests {
    use printer::Print;
    use url::Url;

    use crate::{Arg, Call, CallKind, Exp, Hole, IdBound, MetaVarKind};

    use super::Args;

    #[test]
    fn print_empty_args() {
        let args = Args { args: vec![] };
        assert_eq!(args.print_to_string(Default::default()), "".to_string())
    }

    #[test]
    fn print_unnamed_args() {
        let ctor: Box<Exp> = Box::new(
            Call {
                span: None,
                kind: CallKind::Constructor,
                name: IdBound {
                    span: None,
                    id: "T".to_owned(),
                    uri: Url::parse("inmemory:///scratch.pol").unwrap(),
                },
                args: Args { args: vec![] },
                inferred_type: None,
            }
            .into(),
        );

        assert_eq!(
            Args { args: vec![Arg::UnnamedArg { arg: ctor.clone(), erased: false }] }
                .print_to_string(Default::default()),
            "(T)".to_string()
        );

        assert_eq!(
            Args {
                args: vec![
                    Arg::UnnamedArg { arg: ctor.clone(), erased: false },
                    Arg::UnnamedArg { arg: ctor, erased: false }
                ]
            }
            .print_to_string(Default::default()),
            "(T, T)".to_string()
        )
    }

    #[test]
    fn print_implicit_inserted_args() {
        let hole: Hole = Hole {
            span: None,
            kind: MetaVarKind::Inserted,
            metavar: crate::MetaVar { span: None, kind: crate::MetaVarKind::Inserted, id: 42 },
            inferred_type: None,
            inferred_ctx: None,
            args: vec![],
            solution: None,
        };

        assert_eq!(
            Args { args: vec![Arg::InsertedImplicitArg { hole, erased: false }] }
                .print_to_string(Default::default()),
            "".to_string()
        )
    }
}
