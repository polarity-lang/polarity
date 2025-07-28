use std::collections::HashMap;
use std::convert::Infallible;
use std::fmt::Debug;

use values::Binder;

use crate::Variable;
use crate::ctx::*;
use crate::*;

// New concrete representation of Substitution
//
//

/// # Substitutions as Context Morphisms
///
/// A substitution `θ` is a context morphism
/// ```txt
/// θ : Γ ⇒ Δ
/// ```
/// which has a domain `Γ` and a codomain `Δ`.
/// It can be applied to judgements having a context `Γ` as follows:
/// ```
/// Γ ⊢ J    θ : Γ ⇒ Δ
/// ----------------
///    Δ ⊢ J θ
/// ```
/// ## Domain of a context morphism
///
/// We represent the domain `Γ` using deBruijn levels which allows for automatic weakening, so every substitution:
/// ```txt
/// θ : Γ₁ ⇒ Δ
/// ```
/// is also a valid substitution in an extended context:
/// ```
/// θ : Γ₁, Γ₂  ⇒ Δ
/// ```
///
/// ## Codomain of a context morphism
///
/// The expressions of the substitution have variables which are encoded as deBruijn Indices relative to
/// the codomain `Δ`. So if we want to apply the substitution under a binder we have to shift the expressions contained in it.
/// ```txt
///        θ : Γ ⇒ Δ₁
/// -----------------
/// shift(θ) : Γ ⇒ Δ₁, Δ₂
/// ```
#[derive(Debug, Clone)]
pub struct Subst {
    pub hm: HashMap<Lvl, Exp>,
}

impl Shift for Subst {
    fn shift_in_range<R: ShiftRange>(&mut self, range: &R, by: (isize, isize)) {
        todo!()
    }
}

impl Subst {
    /// Construct a substitution from a deBruijn level and an expression
    ///
    /// # Requires
    ///
    /// - `Δ ⊢ exp : τ
    /// - `lvl` is valid in Γ
    ///
    /// # Ensures
    ///
    /// - `θ ⊢ Γ ⇒ Δ` where `θ` is the output of the function.
    pub fn assign(lvl: Lvl, exp: Exp) -> Self {
        let mut hm = HashMap::new();
        hm.insert(lvl, exp);
        Subst { hm }
    }
}

pub trait SubstitutionNew: Sized {
    type Target;
    /// Apply a substitution to an entity.
    ///
    /// ## Input
    /// - `ctx` This corresponds to `Γ` in the precondition and postcondition.
    /// - `subst` This corresponds to `θ`
    ///
    /// ## Requires
    ///
    /// - `Γ ⊢ J`
    ///   Where `J` corresponds to the judgement form for the self parameter to which we apply the substitution
    /// - `θ : Γ ⇒ Δ`
    ///
    /// ## Ensures
    ///
    /// - `Δ ⊢ J θ` where `J θ` is the return value of the function.
    fn subst_new(&self, ctx: &LevelCtx, subst: &Subst) -> Self::Target;
}

impl<T: SubstitutionNew> SubstitutionNew for Option<T> {
    type Target = Option<T::Target>;
    fn subst_new(&self, ctx: &LevelCtx, subst: &Subst) -> Self::Target {
        self.as_ref().map(|x| x.subst_new(ctx, subst))
    }
}

impl<T: SubstitutionNew> SubstitutionNew for Vec<T> {
    type Target = Vec<T::Target>;
    fn subst_new(&self, ctx: &LevelCtx, subst: &Subst) -> Self::Target {
        self.iter().map(|x| x.subst_new(ctx, subst)).collect::<Vec<_>>()
    }
}

impl<T: SubstitutionNew> SubstitutionNew for Box<T> {
    type Target = Box<T::Target>;
    fn subst_new(&self, ctx: &LevelCtx, subst: &Subst) -> Self::Target {
        Box::new((**self).subst_new(ctx, subst))
    }
}

// Substitution
//
//

/// Trait for entities which can be used as a substitution.
/// In order to be used as a substitution an entity has to provide a method
/// to query it for a result for a given deBruijn Level.
pub trait Substitution: Shift + Clone + Debug {
    type Err;

    fn get_subst(&self, ctx: &LevelCtx, lvl: Lvl) -> Result<Option<Box<Exp>>, Self::Err>;
}

impl Substitution for Vec<Vec<Box<Exp>>> {
    type Err = Infallible;

    fn get_subst(&self, _ctx: &LevelCtx, lvl: Lvl) -> Result<Option<Box<Exp>>, Self::Err> {
        if lvl.fst >= self.len() {
            return Ok(None);
        }
        Ok(Some(self[lvl.fst][lvl.snd].clone()))
    }
}

impl Substitution for Vec<Vec<Binder<Box<Exp>>>> {
    type Err = Infallible;

    fn get_subst(&self, _ctx: &LevelCtx, lvl: Lvl) -> Result<Option<Box<Exp>>, Self::Err> {
        if lvl.fst >= self.len() {
            return Ok(None);
        }
        Ok(Some(self[lvl.fst][lvl.snd].content.clone()))
    }
}

impl Substitution for Vec<Vec<Arg>> {
    type Err = Infallible;

    fn get_subst(&self, _ctx: &LevelCtx, lvl: Lvl) -> Result<Option<Box<Exp>>, Self::Err> {
        if lvl.fst >= self.len() {
            return Ok(None);
        }
        Ok(Some(self[lvl.fst][lvl.snd].exp().clone()))
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
    type Err = Infallible;

    fn get_subst(&self, _ctx: &LevelCtx, lvl: Lvl) -> Result<Option<Box<Exp>>, Self::Err> {
        if self.lvl == lvl { Ok(Some(self.exp.clone())) } else { Ok(None) }
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
    type Target;
    fn subst<S: Substitution>(&self, ctx: &mut LevelCtx, by: &S) -> Result<Self::Target, S::Err>;
}

impl<T: Substitutable> Substitutable for Option<T> {
    type Target = Option<T::Target>;
    fn subst<S: Substitution>(&self, ctx: &mut LevelCtx, by: &S) -> Result<Self::Target, S::Err> {
        self.as_ref().map(|x| x.subst(ctx, by)).transpose()
    }
}

impl<T: Substitutable> Substitutable for Vec<T> {
    type Target = Vec<T::Target>;
    fn subst<S: Substitution>(&self, ctx: &mut LevelCtx, by: &S) -> Result<Self::Target, S::Err> {
        self.iter().map(|x| x.subst(ctx, by)).collect::<Result<Vec<_>, _>>()
    }
}

impl<T: Substitutable> Substitutable for Box<T> {
    type Target = Box<T::Target>;
    fn subst<S: Substitution>(&self, ctx: &mut LevelCtx, by: &S) -> Result<Self::Target, S::Err> {
        Ok(Box::new((**self).subst(ctx, by)?))
    }
}

// SwapWithCtx
//
//

pub trait SwapWithCtx: Substitutable {
    fn swap_with_ctx(&self, ctx: &mut LevelCtx, fst1: usize, fst2: usize) -> Self::Target;
}

impl<T: Substitutable> SwapWithCtx for T {
    fn swap_with_ctx(
        &self,
        ctx: &mut LevelCtx,
        fst1: usize,
        fst2: usize,
    ) -> <T as Substitutable>::Target {
        // Unwrap is safe here because we are unwrapping an infallible result
        self.subst(ctx, &SwapSubst { fst1, fst2 }).unwrap()
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
    type Err = Infallible;

    fn get_subst(&self, ctx: &LevelCtx, lvl: Lvl) -> Result<Option<Box<Exp>>, Self::Err> {
        let new_lvl = if lvl.fst == self.fst1 {
            Some(Lvl { fst: self.fst2, snd: lvl.snd })
        } else if lvl.fst == self.fst2 {
            Some(Lvl { fst: self.fst1, snd: lvl.snd })
        } else {
            None
        };

        let new_ctx = ctx.swap(self.fst1, self.fst2);

        Ok(new_lvl.map(|new_lvl| {
            Box::new(Exp::Variable(Variable {
                span: None,
                idx: new_ctx.lvl_to_idx(new_lvl),
                name: VarBound::from_string(""),
                inferred_type: None,
            }))
        }))
    }
}

// SubstInTelescope
//
//

pub trait SubstInTelescope: Sized {
    /// Substitute in a telescope
    fn subst_in_telescope<S: Substitution>(&self, ctx: LevelCtx, s: &S) -> Result<Self, S::Err>;
}

impl SubstInTelescope for Telescope {
    fn subst_in_telescope<S: Substitution>(
        &self,
        mut ctx: LevelCtx,
        s: &S,
    ) -> Result<Self, S::Err> {
        let Telescope { params } = self;

        ctx.bind_fold_failable(
            params.iter(),
            Vec::new(),
            |ctx, params_out, param| {
                params_out.push(param.subst(ctx, s)?);
                Ok(Binder { name: param.name.clone(), content: () })
            },
            |_, params_out| Telescope { params: params_out },
        )
    }
}
