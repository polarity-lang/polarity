use codespan::Span;
use derivative::Derivative;
use pretty::DocAllocator;
use printer::{
    tokens::{COLONEQ, COMMA},
    Alloc, Builder, Print, PrintCfg,
};

use crate::{
    ctx::LevelCtx, ContainsMetaVars, HasSpan, HasType, Occurs, Shift, ShiftRange, Substitutable,
    Substitution, Zonk, ZonkError,
};

use super::{Exp, Hole, Lvl, MetaVar, VarBound};

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
    NamedArg(VarBound, Box<Exp>),
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
            Args { args: vec![Arg::UnnamedArg(ctor.clone())] }.print_to_string(Default::default()),
            "(T)".to_string()
        );

        assert_eq!(
            Args { args: vec![Arg::UnnamedArg(ctor.clone()), Arg::UnnamedArg(ctor)] }
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
            Args { args: vec![Arg::InsertedImplicitArg(hole)] }.print_to_string(Default::default()),
            "".to_string()
        )
    }
}
