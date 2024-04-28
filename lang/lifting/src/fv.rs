use std::cmp;
use std::rc::Rc;

use derivative::Derivative;

use syntax::common::*;
use syntax::ctx::values::TypeCtx;
use syntax::ctx::*;
use syntax::generic::{Hole, TypeUniv, Variable};
use syntax::ust::{self, Occurs};

/// Find all free variables
pub fn free_vars<T: FV>(arg: &T, ctx: &TypeCtx) -> FreeVars {
    let mut v = USTVisitor {
        fvs: Default::default(),
        cutoff: ctx.len(),
        type_ctx: ctx,
        lvl_ctx: ctx.levels(),
    };

    arg.visit_fv(&mut v);

    FreeVars { fvs: v.fvs, cutoff: ctx.len() }
}

pub trait FV {
    fn visit_fv(&self, v: &mut USTVisitor);
}

impl FV for ust::Exp {
    fn visit_fv(&self, v: &mut USTVisitor) {
        match self {
            ust::Exp::Anno(ust::Anno { exp, typ, .. }) => {
                exp.visit_fv(v);
                typ.visit_fv(v)
            }
            ust::Exp::Variable(e) => e.visit_fv(v),
            ust::Exp::LocalComatch(ust::LocalComatch {
                span: _,
                info: _,
                ctx: _,
                name: _,
                is_lambda_sugar: _,
                body,
            }) => body.visit_fv(v),
            ust::Exp::Call(ust::Call { span: _, info: _, name: _, args }) => args.visit_fv(v),
            ust::Exp::DotCall(ust::DotCall { span: _, info: _, exp, name: _, args }) => {
                exp.visit_fv(v);
                args.visit_fv(v);
            }
            ust::Exp::TypCtor(e) => e.visit_fv(v),
            ust::Exp::Hole(Hole { .. }) => {}
            ust::Exp::TypeUniv(TypeUniv { span: _ }) => {}
            ust::Exp::LocalMatch(ust::LocalMatch {
                span: _,
                info: _,
                ctx: _,
                name: _,
                on_exp,
                motive,
                ret_typ: _,
                body,
            }) => {
                on_exp.visit_fv(v);
                motive.visit_fv(v);
                body.visit_fv(v)
            }
        }
    }
}

impl FV for Variable {
    fn visit_fv(&self, v: &mut USTVisitor) {
        let Variable { idx, name, .. } = self;
        // We use the level context to convert the De Bruijn index to a De Bruijn level
        let lvl = v.lvl_ctx.idx_to_lvl(*idx);
        // If the variable is considered free (based on the cutoff), we look up its type in the typing context
        // The typing context contains the types for all free variables where lvl < cutoff
        if lvl.fst < v.cutoff {
            let typ = v
                .type_ctx
                .lookup(lvl)
                .typ
                .shift(((v.lvl_ctx.len() - v.type_ctx.len()) as isize, 0));
            v.add_fv(name.clone(), lvl, typ, v.lvl_ctx.clone())
        }
    }
}

impl FV for ust::TypCtor {
    fn visit_fv(&self, v: &mut USTVisitor) {
        let ust::TypCtor { span: _, name: _, args } = self;
        args.visit_fv(v)
    }
}

impl FV for ust::Args {
    fn visit_fv(&self, v: &mut USTVisitor) {
        let ust::Args { args } = self;
        for arg in args {
            arg.visit_fv(v)
        }
    }
}

impl FV for ust::Match {
    fn visit_fv(&self, v: &mut USTVisitor) {
        let ust::Match { span: _, cases, omit_absurd: _ } = self;
        for case in cases {
            case.visit_fv(v)
        }
    }
}

impl FV for ust::Case {
    fn visit_fv(&self, v: &mut USTVisitor) {
        let ust::Case { span: _, name: _, params, body } = self;

        v.bind_iter(params.params.iter(), |v| {
            body.visit_fv(v);
        })
    }
}

impl FV for ust::Motive {
    fn visit_fv(&self, v: &mut USTVisitor) {
        let ust::Motive { span: _, param, ret_typ } = self;

        param.visit_fv(v);

        v.bind_single(param, |v| ret_typ.visit_fv(v))
    }
}

impl FV for ust::ParamInst {
    fn visit_fv(&self, _v: &mut USTVisitor) {
        //contains no type info for ust.
    }
}

impl<T: FV> FV for Rc<T> {
    fn visit_fv(&self, v: &mut USTVisitor) {
        (**self).visit_fv(v)
    }
}

impl<T: FV> FV for Option<T> {
    fn visit_fv(&self, v: &mut USTVisitor) {
        if let Some(x) = self {
            x.visit_fv(v)
        }
    }
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
            let arg = Rc::new(ust::Exp::Variable(Variable {
                span: None,
                idx: base_ctx.lvl_to_idx(fv.lvl),
                name: name.clone(),
                inferred_type: None,
            }));
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

#[derive(Clone, Debug, Derivative)]
#[derivative(Hash, Eq, PartialEq)]
pub struct FreeVar {
    /// Name of the free variable
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub name: ust::Ident,
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

/// Visitor that collects free variables in an untyped syntax tree
pub struct USTVisitor<'a> {
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
    fn add_fv(&mut self, name: ust::Ident, lvl: Lvl, typ: Rc<ust::Exp>, ctx: LevelCtx) {
        // Add the free variable
        let fv = FreeVar { name, lvl, typ: typ.clone(), ctx };
        if self.fvs.insert(fv) {
            // If it has not already been added:
            // Find all free variables in the type of the free variable
            typ.visit_fv(self);
        }
    }
}

impl<'a> BindContext for USTVisitor<'a> {
    type Ctx = LevelCtx;

    fn ctx_mut(&mut self) -> &mut Self::Ctx {
        &mut self.lvl_ctx
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
    name: ust::Ident,
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

    fn add(&mut self, name: ust::Ident, lvl: Lvl) {
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
    fn shift_in_range<R: ShiftRange>(&self, _range: R, _by: (isize, isize)) -> Self {
        // Since FVSubst works with levels, it is shift-invariant
        self.clone()
    }
}

impl<'a> Shift for FVBodySubst<'a> {
    fn shift_in_range<R: ShiftRange>(&self, _range: R, _by: (isize, isize)) -> FVBodySubst<'a> {
        // Since FVSubst works with levels, it is shift-invariant
        FVBodySubst { inner: self.inner }
    }
}

impl<'a> Shift for FVParamSubst<'a> {
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
            Rc::new(ust::Exp::Variable(Variable {
                span: None,
                idx: new_ctx.lvl_to_idx(fv.lvl),
                name: fv.name.clone(),
                inferred_type: None,
            }))
        })
    }
}

impl<'a> Substitution<Rc<ust::Exp>> for FVParamSubst<'a> {
    fn get_subst(&self, _ctx: &LevelCtx, lvl: Lvl) -> Option<Rc<ust::Exp>> {
        self.inner.subst.get(&lvl).map(|fv| {
            Rc::new(ust::Exp::Variable(Variable {
                span: None,
                idx: Idx { fst: 0, snd: self.inner.subst.len() - 1 - fv.lvl.snd },
                name: fv.name.clone(),
                inferred_type: None,
            }))
        })
    }
}
