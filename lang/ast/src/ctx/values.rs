//! Variable context
//!
//! Tracks locally bound variables

use crate::traits::Shift;
use crate::*;

use super::{GenericCtx, LevelCtx};

pub type TypeCtx = GenericCtx<Box<Exp>>;

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

    pub fn map_failable<E, F>(&self, f: F) -> Result<Self, E>
    where
        F: Fn(&Exp) -> Result<Box<Exp>, E>,
    {
        let bound: Result<_, _> = self
            .bound
            .iter()
            .map(|stack| {
                stack
                    .iter()
                    .map(|b| Ok(Binder { name: b.name.clone(), content: f(&b.content)? }))
                    .collect()
            })
            .collect();

        Ok(Self { bound: bound? })
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
