use std::cmp;
use std::convert::Infallible;

use ast::ctx::*;
use ast::*;
use ast::{Occurs, Variable};

use super::fv::*;

/// Compute the lifted signature based on the set of free variables of an expression `e`.
/// Using the lifted signature `LiftedSignature { telescope, subst, args }`, the lifted expression
///
/// ```text
/// let f(telescope) { e[subst] }
/// ```
///
/// can be constructed, where `f` is a fresh name. The expression `e` can be replaced by `f(args)`.
///
/// # Parameters
///
/// - `fvs`: Set of free variables
/// - `base_ctx`: Context under which the expression `e` is well-typed
///
/// # Requires
///
/// - `fvs ⊆ base_ctx`
///
/// # Returns
///
/// The signature `LiftedSignature { telescope, subst, args }` of the lifted expression, consisting of:
///
/// - `telescope`: The telescope under which `e[subst]` is closed
/// - `subst`: A substitution that closes the free variables under `telescope`
/// - `args`: The arguments to apply to the lifted expression
///
/// # Ensures
///
/// - `telescope ⊆ base_ctx`
/// - `let f(telescope) { e[subst] }` is well-typed in the empty context
/// - `e = f(args)` and well-typed in `base_ctx`
///
pub fn lifted_signature(fvs: HashSet<FreeVar>, base_ctx: &LevelCtx) -> LiftedSignature {
    let cutoff = base_ctx.len();
    // Sort the list of free variables by the De-Bruijn level such the dependency relation is satisfied.
    // Types can only depend on types which occur earlier in the context.
    let fvs = sort_free_vars(fvs);

    let mut params: Vec<Param> = vec![];
    let mut args = vec![];
    let mut subst = FVParamSubst::new(cutoff);

    for fv in fvs.into_iter() {
        let FreeVar { name, lvl, typ, mut ctx } = fv;

        // Unwrap is safe here because we are unwrapping an infallible result
        let typ = typ.subst(&mut ctx, &subst).unwrap();

        let param = Param {
            implicit: false,
            name: VarBind::from_string(&name),
            typ: typ.clone(),
            erased: false,
        };
        let arg = Arg::UnnamedArg {
            arg: Box::new(Exp::Variable(Variable {
                span: None,
                idx: base_ctx.lvl_to_idx(fv.lvl),
                name: VarBound::from_string(&name),
                inferred_type: None,
            })),
            erased: false,
        };
        args.push(arg);
        params.push(param);
        subst.add(name, lvl);
    }

    LiftedSignature {
        telescope: Telescope { params },
        subst: subst.into_body_subst(),
        args: Args { args },
    }
}

/// The signature of a lifted expression
pub struct LiftedSignature {
    /// Telescope of the lifted expression
    pub telescope: Telescope,
    /// Substitution that is applied to the body of the lifted expression
    pub subst: FVBodySubst,
    /// An instantiation of `telescope` with the free variables
    pub args: Args,
}

/// Sort the free variables such the dependency relation is satisfied
/// Due to unification, it is not sufficient to sort them according to their De-Bruijn level:
/// Unification can lead to a set of free variables where variables with a higher De-Bruijn level
/// may occur in the types of variables with a lower De-Bruijn level.
/// This is because unification may locally refine types.
/// Example:
///
/// ```pol
/// data Bar(a: Type) { }
///
/// codata Baz { unit: Top }
///
/// data Foo(a: Type) {
///    MkFoo(a: Type): Foo(Bar(a)),
/// }
///
/// data Top { Unit }
///
/// def Top.ignore(a: Type, x: a): Top {
///     Unit => Unit
/// }
///
/// def Top.foo(a: Type, foo: Foo(a)): Baz {
///     Unit => foo.match {
///         MkFoo(a') => comatch {
///            unit => Unit.ignore(Foo(Bar(a')), foo)
///        }
///    }
/// }
/// ```
///
/// In this example, unification may perform the substitution `{a := a'}` such that locally
/// the type of foo is known to be `Foo(Bar(a'))`.
/// Hence, lifting of the comatch will need to consider the free variables [ foo: Foo(Bar(a')), a': Type ]
/// where `foo` depends on `a'` even though it has been bound earlier in the context
fn sort_free_vars(fvs: HashSet<FreeVar>) -> Vec<FreeVar> {
    let mut fvs: Vec<_> = fvs.into_iter().collect();
    fvs.sort();
    fvs
}

impl PartialOrd for FreeVar {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FreeVar {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let self_occurs_in_other = other.typ.occurs(&mut other.ctx.clone(), self.lvl);
        let other_occurs_in_self = self.typ.occurs(&mut self.ctx.clone(), other.lvl);
        assert!(!(self_occurs_in_other && other_occurs_in_self));
        if self_occurs_in_other {
            cmp::Ordering::Less
        } else if other_occurs_in_self {
            cmp::Ordering::Greater
        } else {
            self.lvl.cmp(&other.lvl)
        }
    }
}

/// Substitution applied to parameters of the telscope
#[derive(Clone, Debug)]
pub struct FVParamSubst {
    /// Mapping of the original De-Bruijn levels of a free variable to the new reference
    subst: HashMap<Lvl, NewVar>,
    /// The De-Bruijn level (fst index) up to which a variable counts as free
    cutoff: usize,
}

impl FVParamSubst {
    fn new(cutoff: usize) -> Self {
        Self { subst: Default::default(), cutoff }
    }

    fn add(&mut self, name: String, lvl: Lvl) {
        self.subst.insert(lvl, NewVar { name, lvl: Lvl { fst: 0, snd: self.subst.len() } });
    }

    /// Build the substitution applied to the body of the new definition
    fn into_body_subst(self) -> FVBodySubst {
        FVBodySubst { subst: self.subst, cutoff: self.cutoff }
    }
}

/// A free variable as part of `FVSubst`
#[derive(Clone, Debug)]
struct NewVar {
    /// Name of the free variable
    name: String,
    /// New De-Bruijn level
    lvl: Lvl,
}

/// Substitution applied to the body of the new definition
#[derive(Clone, Debug)]
pub struct FVBodySubst {
    /// Mapping of the original De-Bruijn levels of a free variable to the new reference
    subst: HashMap<Lvl, NewVar>,
    /// The De-Bruijn level (fst index) up to which a variable counts as free
    cutoff: usize,
}

impl Shift for FVParamSubst {
    fn shift_in_range<R: ShiftRange>(&mut self, _range: &R, _by: (isize, isize)) {
        // Since FVSubst works with levels, it is shift-invariant
    }
}

impl Shift for FVBodySubst {
    fn shift_in_range<R: ShiftRange>(&mut self, _range: &R, _by: (isize, isize)) {
        // Since FVSubst works with levels, it is shift-invariant
    }
}

impl Substitution for FVBodySubst {
    type Err = Infallible;

    fn get_subst(&self, ctx: &LevelCtx, lvl: Lvl) -> Result<Option<Box<Exp>>, Self::Err> {
        // Let Γ be the original context, let Δ be the context according to which the new De-Bruijn index should be calculated
        //
        // Γ = [[x], [y], [z]]
        //     ^^^^^^^^  ^
        //    free vars  cutoff
        //
        // Δ = [[x, y], [z]]
        //      ^^^^^^  ^^^ bound vars
        // new telescope

        // Compute the names for the free variables in the correct order
        // This is only needed to satisfy LevelCtx now tracking the names of the binders.
        // FIXME: This needs to be refactored
        let mut free_vars = self.subst.iter().collect::<Vec<_>>();
        free_vars.sort_by_key(|(lvl, _)| *lvl);
        let free_vars = free_vars
            .into_iter()
            .map(|(_, var)| VarBind::Var { id: var.name.clone(), span: None })
            .collect::<Vec<_>>();
        let new_ctx = LevelCtx::from(vec![free_vars]).append(&ctx.tail(self.cutoff));
        Ok(self.subst.get(&lvl).map(|fv| {
            Box::new(Exp::Variable(Variable {
                span: None,
                idx: new_ctx.lvl_to_idx(fv.lvl),
                name: VarBound::from_string(&fv.name),
                inferred_type: None,
            }))
        }))
    }
}

impl Substitution for FVParamSubst {
    type Err = Infallible;

    fn get_subst(&self, _ctx: &LevelCtx, lvl: Lvl) -> Result<Option<Box<Exp>>, Self::Err> {
        Ok(self.subst.get(&lvl).map(|fv| {
            Box::new(Exp::Variable(Variable {
                span: None,
                idx: Idx { fst: 0, snd: self.subst.len() - 1 - fv.lvl.snd },
                name: VarBound::from_string(&fv.name),
                inferred_type: None,
            }))
        }))
    }
}
