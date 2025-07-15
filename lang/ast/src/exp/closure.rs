use crate::ctx::LevelCtx;
use crate::ctx::values::Binder;
use crate::rename::Rename;
use crate::{
    ContainsMetaVars, Exp, FreeVars, HashMap, HashSet, Lvl, MetaVar, MetaVarState, Occurs, Shift,
    Substitutable, VarBind, VarBound, Variable, Zonk, ZonkError,
};

/// A closure tracking free variables (and their substitution).
/// This is used in (co)matches.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Closure {
    /// A map from de Bruijn level to substituted expression.
    pub args: Vec<Vec<Binder<Option<Box<Exp>>>>>,
}

/// Given a variable context and a level which is bound in that context compute a binder with
/// corresponding bound variable.
///
/// # Assumes
///
/// - The given `lvl` must be bound in the given `ctx`
///
///  # Ensures
///
/// - The returned Binder contains an expression which is bound in the context.
fn compute_binder(ctx: &LevelCtx, lvl: Lvl, has_content: bool) -> Binder<Option<Box<Exp>>> {
    let var_bind = ctx.bound[lvl.fst][lvl.snd].name.clone();
    let name = match &var_bind {
        VarBind::Var { id, .. } => VarBound { span: None, id: id.clone() },
        // When we encouter a wildcard, we use `x` as a placeholder name for the variable referencing this binder.
        // Of course, `x` is not guaranteed to be unique; in general we do not guarantee that the string representation of variables remains intact during elaboration.
        // When reliable variable names are needed (e.g. for printing source code or code generation), the `renaming` transformation needs to be applied to the AST first.
        VarBind::Wildcard { .. } => VarBound::from_string("x"),
    };
    Binder {
        name: var_bind,
        content: has_content.then(|| {
            let exp = Exp::Variable(Variable {
                span: None,
                idx: ctx.lvl_to_idx(lvl),
                name,
                inferred_type: None,
            });

            Box::new(exp)
        }),
    }
}

impl Closure {
    /// Identity substitution for `ctx` restricted to `free_vars`.
    pub fn identity(ctx: &LevelCtx, free_vars: &HashSet<Lvl>) -> Self {
        let mut args = Vec::with_capacity(ctx.len());
        for fst in 0..ctx.len() {
            let mut inner = Vec::with_capacity(ctx.bound[fst].len());
            for snd in 0..ctx.bound[fst].len() {
                let lvl = Lvl { fst, snd };
                // The closure we compute should only contain binders for variables that occur free in the body.
                let binder = compute_binder(ctx, lvl, free_vars.contains(&lvl));
                inner.push(binder);
            }
            args.push(inner);
        }
        Self { args }
    }
}

impl Substitutable for Closure {
    type Target = Closure;

    fn subst<S: crate::Substitution>(
        &self,
        ctx: &mut crate::ctx::LevelCtx,
        by: &S,
    ) -> Result<Self::Target, S::Err> {
        let new_args = Vec::with_capacity(self.args.len());

        for fst in 0..self.args.len() {
            let mut new_inner = Vec::with_capacity(self.args[fst].len());
            for snd in 0..self.args[fst].len() {
                let old_binder = &self.args[fst][snd];
                let new_binder = Binder {
                    name: old_binder.name.clone(),
                    content: old_binder.content.subst(ctx, by)?,
                };
                new_inner.push(new_binder);
            }
        }

        Ok(Closure { args: new_args })
    }
}

impl Occurs for Closure {
    fn occurs<F>(&self, ctx: &mut LevelCtx, f: &F) -> bool
    where
        F: Fn(&LevelCtx, &Exp) -> bool,
    {
        let Closure { args } = self;

        args.iter().flat_map(|inner| inner.iter()).any(|b| b.occurs(ctx, f))
    }
}

impl Zonk for Closure {
    fn zonk(&mut self, meta_vars: &HashMap<MetaVar, MetaVarState>) -> Result<(), ZonkError> {
        for binder in self.args.iter_mut().flatten() {
            binder.content.zonk(meta_vars)?;
        }
        Ok(())
    }
}

impl ContainsMetaVars for Closure {
    fn contains_metavars(&self) -> bool {
        self.args.iter().flat_map(|inner| inner.iter()).any(|b| b.contains_metavars())
    }
}

impl Shift for Closure {
    fn shift_in_range<R: crate::ShiftRange>(&mut self, range: &R, by: (isize, isize)) {
        for binder in &mut self.args.iter_mut().flatten() {
            binder.content.shift_in_range(range, by);
        }
    }
}

impl Rename for Closure {
    fn rename_in_ctx(&mut self, ctx: &mut crate::rename::RenameCtx) {
        for binder in self.args.iter_mut().flatten() {
            binder.rename_in_ctx(ctx);
        }
    }
}

impl FreeVars for Closure {
    fn free_vars_mut(&self, ctx: &LevelCtx, cutoff: usize, fvs: &mut HashSet<Lvl>) {
        for binder in self.args.iter().flatten() {
            binder.content.free_vars_mut(ctx, cutoff, fvs);
        }
    }
}
