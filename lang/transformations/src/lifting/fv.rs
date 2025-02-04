use std::cmp;

use derivative::Derivative;

use ast::ctx::values::TypeCtx;
use ast::ctx::*;
use ast::*;
use ast::{Hole, Occurs, TypeUniv, Variable};

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

impl FV for Vec<Case> {
    fn visit_fv(&self, v: &mut USTVisitor) {
        for case in self {
            case.visit_fv(v)
        }
    }
}

impl FV for Exp {
    fn visit_fv(&self, v: &mut USTVisitor) {
        match self {
            Exp::Anno(Anno { exp, typ, .. }) => {
                exp.visit_fv(v);
                typ.visit_fv(v)
            }
            Exp::Variable(e) => e.visit_fv(v),
            Exp::LocalComatch(LocalComatch { cases, .. }) => {
                for case in cases {
                    case.visit_fv(v)
                }
            }
            Exp::Call(Call { args, .. }) => args.visit_fv(v),
            Exp::DotCall(DotCall { exp, args, .. }) => {
                exp.visit_fv(v);
                args.visit_fv(v);
            }
            Exp::TypCtor(e) => e.visit_fv(v),
            Exp::Hole(e) => e.visit_fv(v),
            Exp::TypeUniv(TypeUniv { span: _ }) => {}
            Exp::LocalMatch(LocalMatch { on_exp, motive, cases, .. }) => {
                for case in cases {
                    case.visit_fv(v);
                }
                on_exp.visit_fv(v);
                motive.visit_fv(v)
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
            let typ = shift_and_clone(
                &v.type_ctx.lookup(lvl).content,
                ((v.lvl_ctx.len() - v.type_ctx.len()) as isize, 0),
            );
            v.add_fv(name.id.clone(), lvl, typ, v.lvl_ctx.clone())
        }
    }
}

impl FV for TypCtor {
    fn visit_fv(&self, v: &mut USTVisitor) {
        let TypCtor { span: _, name: _, args } = self;
        args.visit_fv(v)
    }
}

impl FV for Args {
    fn visit_fv(&self, v: &mut USTVisitor) {
        let Args { args } = self;
        for arg in args {
            arg.visit_fv(v)
        }
    }
}

impl FV for Arg {
    fn visit_fv(&self, v: &mut USTVisitor) {
        match self {
            Arg::UnnamedArg { arg, .. } => arg.visit_fv(v),
            Arg::NamedArg { arg, .. } => arg.visit_fv(v),
            Arg::InsertedImplicitArg { hole, .. } => hole.visit_fv(v),
        }
    }
}

impl FV for Hole {
    fn visit_fv(&self, v: &mut USTVisitor) {
        let Hole { args, .. } = self;
        for subst in args {
            for exp in subst {
                exp.visit_fv(v);
            }
        }
    }
}

impl FV for Case {
    fn visit_fv(&self, v: &mut USTVisitor) {
        let Case { span: _, pattern, body } = self;

        v.bind_iter(pattern.params.params.iter(), |v| {
            body.visit_fv(v);
        })
    }
}

impl FV for Motive {
    fn visit_fv(&self, v: &mut USTVisitor) {
        let Motive { span: _, param, ret_typ } = self;

        param.visit_fv(v);

        v.bind_single(param, |v| ret_typ.visit_fv(v))
    }
}

impl FV for ParamInst {
    fn visit_fv(&self, _v: &mut USTVisitor) {
        //contains no type info for ust.
    }
}

impl<T: FV> FV for Box<T> {
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
    pub telescope: Telescope,
    /// A substitution close the free variables under `telescope`
    pub subst: FVSubst,
    /// An instantiation of `telescope` with the free variables
    pub args: Args,
}

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

#[derive(Clone, Debug, Derivative)]
#[derivative(Hash, Eq, PartialEq)]
pub struct FreeVar {
    /// Name of the free variable
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub name: String,
    /// The original De-Bruijn level
    pub lvl: Lvl,
    /// Type of the free variable
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub typ: Box<Exp>,
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

impl USTVisitor<'_> {
    /// Add a free variable as well as all free variables its type
    fn add_fv(&mut self, name: String, lvl: Lvl, typ: Box<Exp>, ctx: LevelCtx) {
        // Add the free variable
        let fv = FreeVar { name, lvl, typ: typ.clone(), ctx };
        if self.fvs.insert(fv) {
            // If it has not already been added:
            // Find all free variables in the type of the free variable
            typ.visit_fv(self);
        }
    }
}

impl BindContext for USTVisitor<'_> {
    type Content = ();

    fn ctx_mut(&mut self) -> &mut LevelCtx {
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
