//! Variable context
//!
//! Tracks locally bound variables

use pretty::DocAllocator;
use printer::Print;

use crate::traits::Shift;
use crate::*;

use super::{GenericCtx, LevelCtx};

pub type TypeCtx = GenericCtx<Binding>;

impl TypeCtx {
    pub fn levels(&self) -> LevelCtx {
        let bound: Vec<Vec<_>> = self
            .bound
            .iter()
            .map(|inner| {
                inner.iter().map(|b| Binder { name: b.name.to_owned(), content: () }).collect()
            })
            .collect();
        LevelCtx::from(bound)
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Binder<T> {
    pub name: VarBind,
    pub content: T,
}

impl<T: Shift> Shift for Binder<T> {
    fn shift_in_range<R: ShiftRange>(&mut self, range: &R, by: (isize, isize)) {
        self.content.shift_in_range(range, by);
    }
}

impl<T: Occurs> Occurs for Binder<T> {
    fn occurs<F>(&self, ctx: &mut LevelCtx, f: &F) -> bool
    where
        F: Fn(&LevelCtx, &Exp) -> bool,
    {
        self.content.occurs(ctx, f)
    }
}

impl<T: Substitutable> Substitutable for Binder<T> {
    type Target = Binder<T::Target>;

    fn subst<S: Substitution>(&self, ctx: &mut LevelCtx, by: &S) -> Result<Self::Target, S::Err> {
        Ok(Binder { name: self.name.clone(), content: self.content.subst(ctx, by)? })
    }
}

impl<T: ContainsMetaVars> ContainsMetaVars for Binder<T> {
    fn contains_metavars(&self) -> bool {
        self.content.contains_metavars()
    }
}

impl Print for Binder<Box<Exp>> {
    fn print_prec<'a>(
        &'a self,
        cfg: &printer::PrintCfg,
        alloc: &'a printer::Alloc<'a>,
        prec: printer::Precedence,
    ) -> printer::Builder<'a> {
        let Binder { name, content } = self;

        alloc
            .text(name.to_string())
            .append(alloc.text(":="))
            .append(content.print_prec(cfg, alloc, prec))
    }
}

impl Print for Binder<Binding> {
    fn print_prec<'a>(
        &'a self,
        cfg: &printer::PrintCfg,
        alloc: &'a printer::Alloc<'a>,
        prec: printer::Precedence,
    ) -> printer::Builder<'a> {
        let Binder { name, content: Binding { typ, val } } = self;

        let doc = alloc
            .text(name.to_string())
            .append(alloc.text(": "))
            .append(typ.print_prec(cfg, alloc, prec));

        match val {
            Some(BoundValue::PatternMatching { val }) => {
                doc.append(alloc.text(" := ")).append(val.print_prec(cfg, alloc, prec))
            }
            Some(BoundValue::LetBinding { val }) => {
                doc.append(alloc.text(" := ")).append(val.print_prec(cfg, alloc, prec))
            }
            None => doc,
        }
    }
}

impl Print for Binder<()> {
    fn print<'a>(
        &'a self,
        _cfg: &printer::PrintCfg,
        alloc: &'a printer::Alloc<'a>,
    ) -> printer::Builder<'a> {
        let Binder { name, content: () } = self;

        alloc.text(name.to_string())
    }
}

#[derive(Clone, Debug)]
pub struct Binding {
    /// The type of the variable
    pub typ: Box<Exp>,
    /// If the variable is let-bound or refined by pattern matching, this is the bound value
    pub val: Option<BoundValue>,
}

impl Binding {
    pub fn from_type(typ: Box<Exp>) -> Self {
        Binding { typ, val: None }
    }
}

impl Occurs for Binding {
    fn occurs<F>(&self, ctx: &mut LevelCtx, f: &F) -> bool
    where
        F: Fn(&LevelCtx, &Exp) -> bool,
    {
        self.typ.occurs(ctx, f) || self.val.occurs(ctx, f)
    }
}

impl Shift for Binding {
    fn shift_in_range<R: ShiftRange>(&mut self, range: &R, by: (isize, isize)) {
        let Binding { typ, val } = self;

        typ.shift_in_range(range, by);
        val.shift_in_range(range, by);
    }
}

impl Substitutable for Binding {
    type Target = Binding;

    fn subst<S: Substitution>(&self, ctx: &mut LevelCtx, by: &S) -> Result<Self::Target, S::Err> {
        let Binding { typ, val } = self;

        Ok(Binding { typ: typ.subst(ctx, by)?, val: val.subst(ctx, by)? })
    }
}

impl ContainsMetaVars for Binding {
    fn contains_metavars(&self) -> bool {
        self.typ.contains_metavars() || self.val.contains_metavars()
    }
}

#[derive(Clone, Debug)]
pub enum BoundValue {
    /// The variable was substituted by `val` during pattern matching
    PatternMatching { val: Box<Exp> },
    /// The variable was bound to `val` in a let binding
    LetBinding { val: Box<Exp> },
}

impl Shift for BoundValue {
    fn shift_in_range<R: ShiftRange>(&mut self, range: &R, by: (isize, isize)) {
        match self {
            BoundValue::PatternMatching { val } => val.shift_in_range(range, by),
            BoundValue::LetBinding { val } => val.shift_in_range(range, by),
        }
    }
}

impl Occurs for BoundValue {
    fn occurs<F>(&self, ctx: &mut LevelCtx, f: &F) -> bool
    where
        F: Fn(&LevelCtx, &Exp) -> bool,
    {
        match self {
            BoundValue::PatternMatching { val } => val.occurs(ctx, f),
            BoundValue::LetBinding { val } => val.occurs(ctx, f),
        }
    }
}

impl Substitutable for BoundValue {
    type Target = BoundValue;

    fn subst<S: Substitution>(&self, ctx: &mut LevelCtx, by: &S) -> Result<Self::Target, S::Err> {
        match self {
            BoundValue::PatternMatching { val } => {
                Ok(BoundValue::PatternMatching { val: val.subst(ctx, by)? })
            }
            BoundValue::LetBinding { val } => {
                Ok(BoundValue::LetBinding { val: val.subst(ctx, by)? })
            }
        }
    }
}

impl ContainsMetaVars for BoundValue {
    fn contains_metavars(&self) -> bool {
        match self {
            BoundValue::PatternMatching { val } => val.contains_metavars(),
            BoundValue::LetBinding { val } => val.contains_metavars(),
        }
    }
}
