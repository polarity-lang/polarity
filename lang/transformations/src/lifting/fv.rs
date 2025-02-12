use std::collections::HashSet;

use derivative::Derivative;

use ast::ctx::values::TypeCtx;
use ast::ctx::*;
use ast::*;

/// Compute the closure of free variables in `arg` closed under type dependencies
///
/// In a dependent type system, if a variable `x` is free in `arg` (i.e., `x` ∈ `FV(arg, ctx)`),
/// then the type of `x` might itself contain further free variables, and those variables may
/// also have types with additional free variables, and so on. This function computes this
/// transitive closure of free variables in `arg` under type dependencies.
///
/// We can recursively specify this function as follows:
///
/// ```text
/// free_vars_closure(arg, ctx) = FV(arg, ctx) ∪ { free_vars_closure(t, ctx) | for x: t ∈ FV(arg, ctx) }
/// ```
///
/// where `FV(arg, ctx)` is the syntactic set of free variables in `arg` with respect to the context `ctx`.
///
/// # Parameters
///
/// - `arg`: The expression for which to compute the closure of free variables
/// - `ctx`: The typing context in which `arg` is well-typed
///
/// # Requires
///
/// - `arg` is well-typed in `ctx`.
///
/// # Returns
///
/// The fvs is a set of variables, a subset of `ctx`.
/// This set includes every variable that appears free in `arg` (syntactically) as well as
/// any variables that appear free in the types of those variables, recursively.
///
/// # Ensures
///
/// - `free_vars_closure(arg, ctx) ⊆ ctx`
///
pub fn free_vars_closure<T: FV>(arg: &T, ctx: &TypeCtx) -> FreeVars {
    // Start with the context’s level context and the given type context.
    arg.free_vars_closure(ctx.levels().clone(), ctx.clone())
}

/// A trait for computing the free variables of an AST node (transitive closure).
pub trait FV {
    /// Returns the closure of free variables in `self` that are bound in `type_ctx`.
    ///
    /// # Parameters
    ///
    /// - `lvl_ctx`:  The context that contains all free variables in `self`.
    /// - `type_ctx`: The context that contains the types of free variables of interest.
    ///
    /// # Requires
    ///
    /// - `self` is well-typed in `lvl_ctx`.
    /// - `type_ctx ⊆ lvl_ctx`.
    ///
    /// # Returns
    ///
    /// The closure of free variables in `self` closed under type dependencies with respect to `type_ctx`.
    ///
    /// # Ensures
    ///
    /// - `free_vars_closure(self, lvl_ctx, type_ctx) ⊆ type_ctx`
    fn free_vars_closure(&self, lvl_ctx: LevelCtx, type_ctx: TypeCtx) -> FreeVars;
}

/// A set of free variables.
#[derive(Debug, Default)]
pub struct FreeVars {
    pub fvs: HashSet<FreeVar>,
}

impl FreeVars {
    /// Insert a single `FreeVar` into this set.
    /// Returns `true` if it was newly inserted, or `false` if it was already present.
    pub fn insert(&mut self, fv: FreeVar) -> bool {
        self.fvs.insert(fv)
    }

    /// Compute the union of two free variable sets
    pub fn union(self, other: FreeVars) -> FreeVars {
        let mut fvs = self.fvs;
        fvs.extend(other.fvs);
        Self { fvs }
    }
}

/// Information about a free variable, including its name, level, type, and context.
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

impl FV for Exp {
    fn free_vars_closure(&self, lvl_ctx: LevelCtx, type_ctx: TypeCtx) -> FreeVars {
        match self {
            Exp::Anno(anno) => anno.free_vars_closure(lvl_ctx, type_ctx),
            Exp::Variable(var) => var.free_vars_closure(lvl_ctx, type_ctx),
            Exp::LocalComatch(comatch) => comatch.free_vars_closure(lvl_ctx, type_ctx),
            Exp::Call(call) => call.free_vars_closure(lvl_ctx, type_ctx),
            Exp::DotCall(dot_call) => dot_call.free_vars_closure(lvl_ctx, type_ctx),
            Exp::TypCtor(typ_ctor) => typ_ctor.free_vars_closure(lvl_ctx, type_ctx),
            Exp::Hole(hole) => hole.free_vars_closure(lvl_ctx, type_ctx),
            Exp::TypeUniv(_) => FreeVars::default(),
            Exp::LocalMatch(local_match) => local_match.free_vars_closure(lvl_ctx, type_ctx),
        }
    }
}

impl FV for Anno {
    fn free_vars_closure(&self, lvl_ctx: LevelCtx, type_ctx: TypeCtx) -> FreeVars {
        let Anno { exp, typ, .. } = self;
        FreeVars::default()
            .union(exp.free_vars_closure(lvl_ctx.clone(), type_ctx.clone()))
            .union(typ.free_vars_closure(lvl_ctx, type_ctx))
    }
}

impl FV for Variable {
    fn free_vars_closure(&self, lvl_ctx: LevelCtx, type_ctx: TypeCtx) -> FreeVars {
        let mut fvs = FreeVars::default();
        let Variable { idx, name, .. } = self;
        let lvl = lvl_ctx.idx_to_lvl(*idx);

        // Only consider this variable if it is bound in `type_ctx`.
        if lvl.fst < type_ctx.len() {
            let typ = shift_and_clone(
                &type_ctx.lookup(lvl).content,
                ((lvl_ctx.len() - type_ctx.len()) as isize, 0),
            );
            let fv = FreeVar { name: name.id.clone(), lvl, typ: typ.clone(), ctx: lvl_ctx.clone() };

            // If we inserted a new free variable, we must also union in the FV of its type
            if fvs.insert(fv) {
                return fvs.union(typ.free_vars_closure(lvl_ctx, type_ctx));
            }
        }
        fvs
    }
}

impl FV for LocalComatch {
    fn free_vars_closure(&self, lvl_ctx: LevelCtx, type_ctx: TypeCtx) -> FreeVars {
        self.cases.iter().fold(FreeVars::default(), |acc, c| {
            acc.union(c.free_vars_closure(lvl_ctx.clone(), type_ctx.clone()))
        })
    }
}

impl FV for Call {
    fn free_vars_closure(&self, lvl_ctx: LevelCtx, type_ctx: TypeCtx) -> FreeVars {
        self.args.free_vars_closure(lvl_ctx, type_ctx)
    }
}

impl FV for DotCall {
    fn free_vars_closure(&self, lvl_ctx: LevelCtx, type_ctx: TypeCtx) -> FreeVars {
        FreeVars::default()
            .union(self.exp.free_vars_closure(lvl_ctx.clone(), type_ctx.clone()))
            .union(self.args.free_vars_closure(lvl_ctx, type_ctx))
    }
}

impl FV for TypCtor {
    fn free_vars_closure(&self, lvl_ctx: LevelCtx, type_ctx: TypeCtx) -> FreeVars {
        let TypCtor { args, .. } = self;
        args.free_vars_closure(lvl_ctx, type_ctx)
    }
}

impl FV for Args {
    fn free_vars_closure(&self, lvl_ctx: LevelCtx, type_ctx: TypeCtx) -> FreeVars {
        self.args.iter().fold(FreeVars::default(), |acc, arg| {
            acc.union(arg.free_vars_closure(lvl_ctx.clone(), type_ctx.clone()))
        })
    }
}

impl FV for Arg {
    fn free_vars_closure(&self, lvl_ctx: LevelCtx, type_ctx: TypeCtx) -> FreeVars {
        match self {
            Arg::UnnamedArg { arg, .. } => arg.free_vars_closure(lvl_ctx, type_ctx),
            Arg::NamedArg { arg, .. } => arg.free_vars_closure(lvl_ctx, type_ctx),
            Arg::InsertedImplicitArg { hole, .. } => hole.free_vars_closure(lvl_ctx, type_ctx),
        }
    }
}

impl FV for Hole {
    fn free_vars_closure(&self, lvl_ctx: LevelCtx, type_ctx: TypeCtx) -> FreeVars {
        self.args.iter().fold(FreeVars::default(), |acc, subst_list| {
            subst_list.iter().fold(acc, |acc, exp| {
                acc.union(exp.free_vars_closure(lvl_ctx.clone(), type_ctx.clone()))
            })
        })
    }
}

impl FV for LocalMatch {
    fn free_vars_closure(&self, lvl_ctx: LevelCtx, type_ctx: TypeCtx) -> FreeVars {
        let LocalMatch { on_exp, motive, cases, .. } = self;
        let cases_fvs = cases.iter().fold(FreeVars::default(), |acc, c| {
            acc.union(c.free_vars_closure(lvl_ctx.clone(), type_ctx.clone()))
        });
        cases_fvs
            .union(on_exp.free_vars_closure(lvl_ctx.clone(), type_ctx.clone()))
            .union(motive.free_vars_closure(lvl_ctx, type_ctx))
    }
}

impl FV for Case {
    fn free_vars_closure(&self, mut lvl_ctx: LevelCtx, type_ctx: TypeCtx) -> FreeVars {
        let Case { span: _, pattern, body } = self;

        lvl_ctx.bind_iter(pattern.params.params.iter(), |ctx| {
            body.free_vars_closure(ctx.clone(), type_ctx.clone())
        })
    }
}

impl FV for Motive {
    fn free_vars_closure(&self, mut lvl_ctx: LevelCtx, type_ctx: TypeCtx) -> FreeVars {
        let Motive { span: _, param, ret_typ } = self;

        lvl_ctx.bind_iter(std::slice::from_ref(param).iter(), |ctx| {
            ret_typ.free_vars_closure(ctx.clone(), type_ctx.clone())
        })
    }
}

impl FV for ParamInst {
    fn free_vars_closure(&self, _lvl_ctx: LevelCtx, _type_ctx: TypeCtx) -> FreeVars {
        // ParamInst contains no type info relevant for free variables
        FreeVars::default()
    }
}

impl<T: FV> FV for Box<T> {
    fn free_vars_closure(&self, lvl_ctx: LevelCtx, type_ctx: TypeCtx) -> FreeVars {
        (**self).free_vars_closure(lvl_ctx, type_ctx)
    }
}

impl<T: FV> FV for Option<T> {
    fn free_vars_closure(&self, lvl_ctx: LevelCtx, type_ctx: TypeCtx) -> FreeVars {
        if let Some(x) = self {
            x.free_vars_closure(lvl_ctx, type_ctx)
        } else {
            FreeVars::default()
        }
    }
}

impl<T: FV> FV for Vec<T> {
    fn free_vars_closure(&self, lvl_ctx: LevelCtx, type_ctx: TypeCtx) -> FreeVars {
        self.iter().fold(FreeVars::default(), |acc, item| {
            acc.union(item.free_vars_closure(lvl_ctx.clone(), type_ctx.clone()))
        })
    }
}
