use std::fmt::Debug;

use values::Binder;

use crate::ctx::*;
use crate::Variable;
use crate::*;

// Substitution
//
//

/// Trait for entities which can be used as a substitution.
/// In order to be used as a substitution an entity has to provide a method
/// to query it for a result for a given deBruijn Level.
pub trait Substitution: Shift + Clone + Debug {
    fn get_subst(&self, ctx: &LevelCtx, lvl: Lvl) -> Option<Box<Exp>>;
}

impl Substitution for Vec<Vec<Box<Exp>>> {
    fn get_subst(&self, _ctx: &LevelCtx, lvl: Lvl) -> Option<Box<Exp>> {
        if lvl.fst >= self.len() {
            return None;
        }
        Some(self[lvl.fst][lvl.snd].clone())
    }
}

impl Substitution for Vec<Vec<Arg>> {
    fn get_subst(&self, _ctx: &LevelCtx, lvl: Lvl) -> Option<Box<Exp>> {
        if lvl.fst >= self.len() {
            return None;
        }
        Some(self[lvl.fst][lvl.snd].exp().clone())
    }
}

// Assign
//
//

/// An assignment is the simplest form of a substitution which provides just
/// one mapping from a variable (represented by a DeBruijn Level) to an expression.
#[derive(Clone, Debug)]
pub struct Assign {
    pub lvl: Lvl,
    pub exp: Box<Exp>,
}

impl Shift for Assign {
    fn shift_in_range<R: ShiftRange>(&mut self, range: &R, by: (isize, isize)) {
        self.exp.shift_in_range(range, by);
    }
}

impl Substitution for Assign {
    fn get_subst(&self, _ctx: &LevelCtx, lvl: Lvl) -> Option<Box<Exp>> {
        if self.lvl == lvl {
            Some(self.exp.clone())
        } else {
            None
        }
    }
}

// Substitutable
//
//

/// A trait for all entities to which we can apply a substitution.
/// Every syntax node should implement this trait.
/// The result type of applying a substitution is parameterized, because substituting for
/// a variable does not, in general, yield another variable.
pub trait Substitutable: Sized {
    type Result;
    fn subst<S: Substitution>(&self, ctx: &mut LevelCtx, by: &S) -> Self::Result;
}

impl<T: Substitutable> Substitutable for Option<T> {
    type Result = Option<T::Result>;
    fn subst<S: Substitution>(&self, ctx: &mut LevelCtx, by: &S) -> Self::Result {
        self.as_ref().map(|x| x.subst(ctx, by))
    }
}

impl<T: Substitutable> Substitutable for Vec<T> {
    type Result = Vec<T::Result>;
    fn subst<S: Substitution>(&self, ctx: &mut LevelCtx, by: &S) -> Self::Result {
        self.iter().map(|x| x.subst(ctx, by)).collect()
    }
}

impl<T: Substitutable> Substitutable for Box<T> {
    type Result = Box<T::Result>;
    fn subst<S: Substitution>(&self, ctx: &mut LevelCtx, by: &S) -> Self::Result {
        Box::new((**self).subst(ctx, by))
    }
}

// SwapWithCtx
//
//

pub trait SwapWithCtx: Substitutable {
    fn swap_with_ctx(&self, ctx: &mut LevelCtx, fst1: usize, fst2: usize) -> Self::Result;
}

impl<T: Substitutable> SwapWithCtx for T {
    fn swap_with_ctx(
        &self,
        ctx: &mut LevelCtx,
        fst1: usize,
        fst2: usize,
    ) -> <T as Substitutable>::Result {
        self.subst(ctx, &SwapSubst { fst1, fst2 })
    }
}

// SwapSubst
//
//

#[derive(Clone, Debug)]
struct SwapSubst {
    fst1: usize,
    fst2: usize,
}

impl Shift for SwapSubst {
    fn shift_in_range<R: ShiftRange>(&mut self, _range: &R, _by: (isize, isize)) {
        // Since SwapSubst works with levels, it is shift-invariant
    }
}

impl Substitution for SwapSubst {
    fn get_subst(&self, ctx: &LevelCtx, lvl: Lvl) -> Option<Box<Exp>> {
        let new_lvl = if lvl.fst == self.fst1 {
            Some(Lvl { fst: self.fst2, snd: lvl.snd })
        } else if lvl.fst == self.fst2 {
            Some(Lvl { fst: self.fst1, snd: lvl.snd })
        } else {
            None
        };

        let new_ctx = ctx.swap(self.fst1, self.fst2);

        new_lvl.map(|new_lvl| {
            Box::new(Exp::Variable(Variable {
                span: None,
                idx: new_ctx.lvl_to_idx(new_lvl),
                name: VarBound::from_string(""),
                inferred_type: None,
            }))
        })
    }
}

// SubstUnderCtx
//
//

pub trait SubstUnderCtx: Substitutable {
    fn subst_under_ctx<S: Substitution>(&self, ctx: LevelCtx, s: &S) -> Self::Result;
}

impl<T: Substitutable + Clone> SubstUnderCtx for T {
    fn subst_under_ctx<S: Substitution>(&self, mut ctx: LevelCtx, s: &S) -> Self::Result {
        self.subst(&mut ctx, s)
    }
}

// SubstInTelescope
//
//

pub trait SubstInTelescope {
    /// Substitute in a telescope
    fn subst_in_telescope<S: Substitution>(&self, ctx: LevelCtx, s: &S) -> Self;
}

impl SubstInTelescope for Telescope {
    fn subst_in_telescope<S: Substitution>(&self, mut ctx: LevelCtx, s: &S) -> Self {
        let Telescope { params } = self;

        ctx.bind_fold(
            params.iter(),
            Vec::new(),
            |ctx, params_out, param| {
                params_out.push(param.subst(ctx, s));
                BindElem { elem: Binder { name: param.name.clone(), content: () } }
            },
            |_, params_out| Telescope { params: params_out },
        )
    }
}
