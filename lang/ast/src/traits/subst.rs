use std::fmt::Debug;

use values::Binder;

use crate::HashMap;
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
/// ```txt
/// Γ ⊢ J    θ : Γ ⇒ Δ
/// ----------------
///    Δ ⊢ J θ
/// ```
/// ## Domain of a context morphism
///
/// We represent the domain `Γ` using de Bruijn levels which allows for automatic weakening, so every substitution:
/// ```txt
/// θ : Γ₁ ⇒ Δ
/// ```
/// is also a valid substitution in an extended context:
/// ```txt
/// θ : Γ₁, Γ₂  ⇒ Δ
/// ```
///
/// ## Codomain of a context morphism
///
/// The expressions of the substitution have variables which are encoded as de Bruijn Indices relative to
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
        for (_, exp) in self.hm.iter_mut() {
            exp.shift_in_range(range, by);
        }
    }
}

impl Subst {
    /// Construct a substitution from a de Bruijn level and an expression
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
        let mut hm = HashMap::default();
        hm.insert(lvl, exp);
        Subst { hm }
    }

    pub fn from_exps(exps: &[Vec<Box<Exp>>]) -> Self {
        let mut hm: HashMap<Lvl, Exp> = HashMap::default();
        for (fst, vec) in exps.iter().enumerate() {
            for (snd, exp) in vec.iter().enumerate() {
                hm.insert(Lvl { fst, snd }, *exp.clone());
            }
        }
        Subst { hm }
    }

    pub fn from_args(args: &[Vec<Arg>]) -> Self {
        let mut hm: HashMap<Lvl, Exp> = HashMap::default();
        for (fst, vec) in args.iter().enumerate() {
            for (snd, arg) in vec.iter().enumerate() {
                hm.insert(Lvl { fst, snd }, *arg.exp());
            }
        }
        Subst { hm }
    }

    pub fn from_binders(binders: &[Vec<Binder<Box<Exp>>>]) -> Self {
        let mut hm: HashMap<Lvl, Exp> = HashMap::default();
        for (fst, vec) in binders.iter().enumerate() {
            for (snd, binder) in vec.iter().enumerate() {
                hm.insert(Lvl { fst, snd }, *binder.content.clone());
            }
        }
        Subst { hm }
    }

    /// Build a substitution that swaps all variables at de Bruijn levels
    /// with first dimension `fst1` and `fst2`, preserving the second dimension
    /// and leaving all other levels unmapped.
    pub fn swap(ctx: &LevelCtx, fst1: usize, fst2: usize) -> Self {
        let new_ctx = ctx.swap(fst1, fst2);

        let make_var = |lvl: Lvl| {
            Exp::Variable(Variable {
                span: None,
                idx: new_ctx.lvl_to_idx(lvl),
                name: VarBound::from_string(""),
                inferred_type: None,
            })
        };

        let len1 = ctx.bound[fst1].len();
        let len2 = ctx.bound[fst2].len();

        let mut hm = HashMap::default();

        for snd in 0..len1 {
            let from = Lvl { fst: fst1, snd };
            let to = Lvl { fst: fst2, snd };
            hm.insert(from, make_var(to));
        }

        for snd in 0..len2 {
            let from = Lvl { fst: fst2, snd };
            let to = Lvl { fst: fst1, snd };
            hm.insert(from, make_var(to));
        }

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

pub trait SwapWithCtx: SubstitutionNew {
    fn swap_with_ctx(&self, ctx: &mut LevelCtx, fst1: usize, fst2: usize) -> Self::Target;
}

impl<T: SubstitutionNew> SwapWithCtx for T {
    fn swap_with_ctx(
        &self,
        ctx: &mut LevelCtx,
        fst1: usize,
        fst2: usize,
    ) -> <T as SubstitutionNew>::Target {
        self.subst_new(ctx, &Subst::swap(ctx, fst1, fst2))
    }
}
