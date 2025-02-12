use std::cmp;

use ast::ctx::*;
use ast::*;
use ast::{Occurs, Variable};

use super::fv::*;

impl FreeVars {
    pub fn telescope(self, base_ctx: &LevelCtx) -> FreeVarsResult {
        let cutoff = self.cutoff;
        // Sort the list of free variables by the De-Bruijn level such the dependency relation is satisfied.
        // Types can only depend on types which occur earlier in the context.
        let fvs = self.sorted();

        let mut params: Vec<Param> = vec![];
        let mut args = vec![];
        let mut subst = FVSubst::new(cutoff);

        // FIXME: The manual context management here should be abstracted out
        for fv in fvs.into_iter() {
            let FreeVar { name, lvl, typ, mut ctx } = fv;

            let typ = typ.subst(&mut ctx, &subst.in_param());

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

        FreeVarsResult { telescope: Telescope { params }, subst, args: Args { args } }
    }

    /// Compute the union of two free variable sets
    pub fn union(self, other: FreeVars) -> FreeVars {
        assert_eq!(self.cutoff, other.cutoff);
        let mut fvs = self.fvs;
        fvs.extend(other.fvs);
        Self { fvs, cutoff: self.cutoff }
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
    fn sorted(self) -> Vec<FreeVar> {
        let mut fvs: Vec<_> = self.fvs.into_iter().collect();
        fvs.sort();
        fvs
    }
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

/// Substitution of free variables
#[derive(Clone, Debug)]
pub struct FVSubst {
    /// Mapping of the original De-Bruijn levels of a free variable to the new reference
    subst: HashMap<Lvl, NewVar>,
    /// The De-Bruijn level (fst index) up to which a variable counts as free
    cutoff: usize,
}

/// A free variable as part of `FVSubst`
#[derive(Clone, Debug)]
struct NewVar {
    /// Name of the free variable
    name: String,
    /// New De-Bruijn level
    lvl: Lvl,
}

/// Substitution in the body of the new definition
#[derive(Clone, Debug)]
pub struct FVBodySubst<'a> {
    inner: &'a FVSubst,
}

/// Substitution in the type parameters of the new definition
#[derive(Clone, Debug)]
pub struct FVParamSubst<'a> {
    inner: &'a FVSubst,
}

impl FVSubst {
    fn new(cutoff: usize) -> Self {
        Self { subst: Default::default(), cutoff }
    }

    fn add(&mut self, name: String, lvl: Lvl) {
        self.subst.insert(lvl, NewVar { name, lvl: Lvl { fst: 0, snd: self.subst.len() } });
    }

    pub fn in_body(&self) -> FVBodySubst<'_> {
        FVBodySubst { inner: self }
    }

    pub fn in_param(&self) -> FVParamSubst<'_> {
        FVParamSubst { inner: self }
    }
}

impl Shift for FVSubst {
    fn shift_in_range<R: ShiftRange>(&mut self, _range: &R, _by: (isize, isize)) {
        // Since FVSubst works with levels, it is shift-invariant
    }
}

impl Shift for FVBodySubst<'_> {
    fn shift_in_range<R: ShiftRange>(&mut self, _range: &R, _by: (isize, isize)) {
        // Since FVSubst works with levels, it is shift-invariant
    }
}

impl Shift for FVParamSubst<'_> {
    fn shift_in_range<R: ShiftRange>(&mut self, _range: &R, _by: (isize, isize)) {
        // Since FVSubst works with levels, it is shift-invariant
    }
}

impl Substitution for FVBodySubst<'_> {
    fn get_subst(&self, ctx: &LevelCtx, lvl: Lvl) -> Option<Box<Exp>> {
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
        let mut free_vars = self.inner.subst.iter().collect::<Vec<_>>();
        free_vars.sort_by_key(|(lvl, _)| *lvl);
        let free_vars = free_vars
            .into_iter()
            .map(|(_, var)| VarBind::Var { id: var.name.clone(), span: None })
            .collect::<Vec<_>>();
        let new_ctx = LevelCtx::from(vec![free_vars]).append(&ctx.tail(self.inner.cutoff));
        self.inner.subst.get(&lvl).map(|fv| {
            Box::new(Exp::Variable(Variable {
                span: None,
                idx: new_ctx.lvl_to_idx(fv.lvl),
                name: VarBound::from_string(&fv.name),
                inferred_type: None,
            }))
        })
    }
}

impl Substitution for FVParamSubst<'_> {
    fn get_subst(&self, _ctx: &LevelCtx, lvl: Lvl) -> Option<Box<Exp>> {
        self.inner.subst.get(&lvl).map(|fv| {
            Box::new(Exp::Variable(Variable {
                span: None,
                idx: Idx { fst: 0, snd: self.inner.subst.len() - 1 - fv.lvl.snd },
                name: VarBound::from_string(&fv.name),
                inferred_type: None,
            }))
        })
    }
}
