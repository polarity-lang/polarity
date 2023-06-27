use std::cmp;
use std::rc::Rc;

use derivative::Derivative;

use data::{HashMap, HashSet};

use parser::cst::Ident;
use syntax::common::*;
use syntax::ctx::values::TypeCtx;
use syntax::ctx::*;
use syntax::generic::{Visit, Visitor};
use syntax::ust::{self, Occurs};

/// Find all free variables
pub trait FreeVarsExt {
    fn free_vars(&self, ctx: &TypeCtx) -> FreeVars;
}

#[derive(Debug)]
pub struct FreeVars {
    /// The De-Bruijn level (fst index) up to which a variable counts as free
    cutoff: usize,
    /// List of found free variables
    fvs: HashSet<FreeVar>,
}

/// The result of closing under the set of free variables
pub struct FreeVarsResult {
    /// Telescope of the types of the free variables
    pub telescope: ust::Telescope,
    /// A substitution close the free variables under `telescope`
    pub subst: FVSubst,
    /// An instantiation of `telescope` with the free variables
    pub args: ust::Args,
}

impl FreeVars {
    pub fn telescope(self, base_ctx: &LevelCtx) -> FreeVarsResult {
        let cutoff = self.cutoff;
        // Sort the list of free variables by the De-Bruijn level such the dependency relation is satisfied.
        // Types can only depend on types which occur earlier in the context.
        let fvs = self.sorted();

        let mut params: Vec<ust::Param> = vec![];
        let mut args = vec![];
        let mut subst = FVSubst::new(cutoff);

        // FIXME: The manual context management here should be abstracted out
        for fv in fvs.into_iter() {
            let FreeVar { name, lvl, typ, mut ctx } = fv;

            let typ = typ.subst(&mut ctx, &subst.in_param());

            let param = ust::Param { name: name.clone(), typ: typ.clone() };
            let arg = Rc::new(ust::Exp::Var {
                info: Default::default(),
                name: name.clone(),
                ctx: (),
                idx: base_ctx.lvl_to_idx(fv.lvl),
            });
            args.push(arg);
            params.push(param);
            subst.add(name, lvl);
        }

        FreeVarsResult { telescope: ust::Telescope { params }, subst, args: ust::Args { args } }
    }

    /// Compute the union of two free variable sets
    pub fn union(self, other: FreeVars) -> FreeVars {
        assert_eq!(self.cutoff, other.cutoff);
        let mut fvs = self.fvs;
        fvs.extend(other.fvs.into_iter());
        Self { fvs, cutoff: self.cutoff }
    }

    /// Sort the free variables such the dependency relation is satisfied
    /// Due to unification, it is not sufficient to sort them according to their De-Bruijn level:
    /// Unification can lead to a set of free variables where variables with a higher De-Bruijn level
    /// may occur in the types of variables with a lower De-Bruijn level.
    /// This is because unification may locally refine types.
    /// Example:
    ///
    /// ```xfn
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

#[derive(Clone, Debug, Derivative)]
#[derivative(Hash, Eq, PartialEq)]
pub struct FreeVar {
    /// Name of the free variable
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub name: Ident,
    /// The original De-Bruijn level
    pub lvl: Lvl,
    /// Type of the free variable
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub typ: Rc<ust::Exp>,
    /// Context under which the type is closed
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub ctx: LevelCtx,
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

impl<T: Visit<ust::UST>> FreeVarsExt for T {
    fn free_vars(&self, type_ctx: &TypeCtx) -> FreeVars {
        let mut v = USTVisitor {
            fvs: Default::default(),
            cutoff: type_ctx.len(),
            type_ctx,
            lvl_ctx: type_ctx.levels(),
        };

        self.visit(&mut v);

        FreeVars { fvs: v.fvs, cutoff: type_ctx.len() }
    }
}

/// Visitor that collects free variables in an untyped syntax tree
struct USTVisitor<'a> {
    /// Set of collected free variables
    fvs: HashSet<FreeVar>,
    /// The De-Bruijn level (fst index) up to which a variable counts as free
    cutoff: usize,
    /// The typing context where all free variables with lvl < cutoff can be looked up
    type_ctx: &'a TypeCtx,
    /// The level context which tracks the number of binders currently in scope
    lvl_ctx: LevelCtx,
}

impl<'a> USTVisitor<'a> {
    /// Add a free variable as well as all free variables its type
    fn add_fv(&mut self, name: Ident, lvl: Lvl, typ: Rc<ust::Exp>, ctx: LevelCtx) {
        // Add the free variable
        let fv = FreeVar { name, lvl, typ: typ.clone(), ctx };
        if self.fvs.insert(fv) {
            // If it has not already been added:
            // Find all free variables in the type of the free variable
            typ.visit(self);
        }
    }
}

impl<'a> BindContext for USTVisitor<'a> {
    type Ctx = LevelCtx;

    fn ctx_mut(&mut self) -> &mut Self::Ctx {
        &mut self.lvl_ctx
    }
}

impl<'b> Visitor<ust::UST> for USTVisitor<'b> {
    fn visit_telescope<'a, I, F1, F2>(&mut self, params: I, f_acc: F1, f_inner: F2)
    where
        I: IntoIterator<Item = &'a ust::Param>,
        F1: Fn(&mut Self, &'a ust::Param),
        F2: FnOnce(&mut Self),
    {
        self.ctx_visit_telescope(params, f_acc, f_inner)
    }

    fn visit_telescope_inst<'a, I, F1, F2>(&mut self, params: I, f_acc: F1, f_inner: F2)
    where
        I: IntoIterator<Item = &'a ust::ParamInst>,
        F1: Fn(&mut Self, &'a ust::ParamInst),
        F2: FnOnce(&mut Self),
    {
        self.ctx_visit_telescope_inst(params, f_acc, f_inner)
    }

    fn visit_motive_param<X, F>(&mut self, param: &ust::ParamInst, f_inner: F) -> X
    where
        F: FnOnce(&mut Self, &ust::ParamInst) -> X,
    {
        self.ctx_visit_motive_param(param, f_inner)
    }

    fn visit_self_param<X, F>(
        &mut self,
        info: &ust::Info,
        name: &Option<Ident>,
        typ: &ust::TypApp,
        f_inner: F,
    ) -> X
    where
        F: FnOnce(&mut Self) -> X,
    {
        self.ctx_visit_self_param(info, name, typ, f_inner)
    }

    fn visit_exp_var(&mut self, _info: &ust::Info, name: &Ident, _ctx: &(), idx: &Idx) {
        // We use the level context to convert the De Bruijn index to a De Bruijn level
        let lvl = self.lvl_ctx.idx_to_lvl(*idx);
        // If the variable is considered free (based on the cutoff), we look up its type in the typing context
        // The typing context contains the types for all free variables where lvl < cutoff
        if lvl.fst < self.cutoff {
            let typ = self
                .type_ctx
                .lookup(lvl)
                .typ
                .forget()
                .shift(((self.lvl_ctx.len() - self.type_ctx.len()) as isize, 0));
            self.add_fv(name.clone(), lvl, typ, self.lvl_ctx.clone())
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
    name: Ident,
    /// New De-Bruijn level
    lvl: Lvl,
}

/// Substitution in the body of the new definition
pub struct FVBodySubst<'a> {
    inner: &'a FVSubst,
}

/// Substitution in the type parameters of the new definition
pub struct FVParamSubst<'a> {
    inner: &'a FVSubst,
}

impl FVSubst {
    fn new(cutoff: usize) -> Self {
        Self { subst: Default::default(), cutoff }
    }

    fn add(&mut self, name: Ident, lvl: Lvl) {
        self.subst.insert(lvl, NewVar { name, lvl: Lvl { fst: 0, snd: self.subst.len() } });
    }

    pub fn in_body(&self) -> FVBodySubst<'_> {
        FVBodySubst { inner: self }
    }

    pub fn in_param(&self) -> FVParamSubst<'_> {
        FVParamSubst { inner: self }
    }
}

impl ShiftInRange for FVSubst {
    fn shift_in_range<R: ShiftRange>(&self, _range: R, _by: (isize, isize)) -> Self {
        // Since FVSubst works with levels, it is shift-invariant
        self.clone()
    }
}

impl<'a> ShiftInRange for FVBodySubst<'a> {
    fn shift_in_range<R: ShiftRange>(&self, _range: R, _by: (isize, isize)) -> FVBodySubst<'a> {
        // Since FVSubst works with levels, it is shift-invariant
        FVBodySubst { inner: self.inner }
    }
}

impl<'a> ShiftInRange for FVParamSubst<'a> {
    fn shift_in_range<R: ShiftRange>(&self, _range: R, _by: (isize, isize)) -> FVParamSubst<'a> {
        // Since FVSubst works with levels, it is shift-invariant
        FVParamSubst { inner: self.inner }
    }
}

impl<'a> Substitution<Rc<ust::Exp>> for FVBodySubst<'a> {
    fn get_subst(&self, ctx: &LevelCtx, lvl: Lvl) -> Option<Rc<ust::Exp>> {
        // Let Γ be the original context, let Δ be the context according to which the new De-Bruijn index should be calculated
        //
        // Γ = [[x], [y], [z]]
        //     ^^^^^^^^  ^
        //    free vars  cutoff
        //
        // Δ = [[x, y], [z]]
        //      ^^^^^^  ^^^ bound vars
        // new telescope
        let new_ctx =
            LevelCtx::from(vec![self.inner.subst.len()]).append(&ctx.tail(self.inner.cutoff));
        self.inner.subst.get(&lvl).map(|fv| {
            Rc::new(ust::Exp::Var {
                info: Default::default(),
                name: fv.name.clone(),
                ctx: (),
                idx: new_ctx.lvl_to_idx(fv.lvl),
            })
        })
    }
}

impl<'a> Substitution<Rc<ust::Exp>> for FVParamSubst<'a> {
    fn get_subst(&self, _ctx: &LevelCtx, lvl: Lvl) -> Option<Rc<ust::Exp>> {
        self.inner.subst.get(&lvl).map(|fv| {
            Rc::new(ust::Exp::Var {
                info: Default::default(),
                name: fv.name.clone(),
                ctx: (),
                idx: Idx { fst: 0, snd: self.inner.subst.len() - 1 - fv.lvl.snd },
            })
        })
    }
}
