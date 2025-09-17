use derivative::Derivative;
use miette_util::codespan::Span;
use pretty::DocAllocator;
use printer::{
    Alloc, Builder, Precedence, Print, PrintCfg,
    theme::ThemeExt,
    tokens::{COMMA, QUESTION_MARK, UNDERSCORE},
};

use crate::{
    ContainsMetaVars, FreeVars, HasSpan, HasType, Occurs, Shift, ShiftRange, Subst, Substitutable,
    Zonk, ZonkError,
    ctx::{
        LevelCtx,
        values::{Binder, TypeCtx},
    },
    rename::{Rename, RenameCtx},
};

use super::{Exp, MetaVar, MetaVarKind};

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Hole {
    /// Source code location
    pub span: Option<Span>,
    /// Whether the hole must be solved during typechecking or not.
    pub kind: MetaVarKind,
    /// The metavariable that we want to solve for that hole
    pub metavar: MetaVar,
    /// The inferred type of the hole annotated during elaboration.
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub inferred_type: Option<Box<Exp>>,
    /// The type context in which the hole occurs.
    /// This context is annotated during elaboration.
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub inferred_ctx: Option<TypeCtx>,
    /// When a hole is lowered, we apply it to all variables available in the
    /// context, since intuitively, a hole stands for an unknown term which can use
    /// any of these variables.
    /// Some other implementations use a functional application to all variables instead,
    /// but since we do not have a function type we have to use an explicit substitution.
    /// Since our system uses 2-level De-Bruijn indices, the explicit substitution id_Г
    /// is a nested vector.
    ///
    /// Example:
    /// `[x, y][z][v, w] |- ?[x, y][z][v,w]`
    pub args: Vec<Vec<Binder<Box<Exp>>>>,
    /// The solution found by unification. It is propagated during zonking.
    pub solution: Option<Box<Exp>>,
}

impl Hole {
    /// The context of the hole arguments
    pub fn levels(&self) -> LevelCtx {
        let bound = self
            .args
            .iter()
            .map(|args| {
                args.iter()
                    .map(|binder| Binder { name: binder.name.clone(), content: () })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        LevelCtx { bound }
    }
}

impl HasSpan for Hole {
    fn span(&self) -> Option<Span> {
        self.span
    }
}

impl From<Hole> for Exp {
    fn from(val: Hole) -> Self {
        Exp::Hole(val)
    }
}

impl Shift for Hole {
    fn shift_in_range<R: ShiftRange>(&mut self, range: &R, by: (isize, isize)) {
        let Hole { span: _, kind: _, metavar: _, inferred_type, inferred_ctx, args, solution } =
            self;

        *inferred_type = None;
        *inferred_ctx = None;
        args.shift_in_range(range, by);
        solution.shift_in_range(range, by);
    }
}

impl Occurs for Hole {
    fn occurs<F>(&self, ctx: &mut LevelCtx, f: &F) -> bool
    where
        F: Fn(&LevelCtx, &Exp) -> bool,
    {
        let Hole { args, solution, .. } = self;
        args.iter().any(|arg_group| arg_group.iter().any(|arg| arg.occurs(ctx, f)))
            || solution.as_ref().is_some_and(|sol| sol.occurs(ctx, f))
    }
}

impl HasType for Hole {
    fn typ(&self) -> Option<Box<Exp>> {
        self.inferred_type.clone()
    }
}

impl Substitutable for Hole {
    type Target = Hole;

    fn subst(&self, ctx: &mut LevelCtx, subst: &Subst) -> Self::Target {
        let Hole { span, kind, metavar, args, .. } = self;
        Hole {
            span: *span,
            kind: *kind,
            metavar: *metavar,
            inferred_type: None,
            inferred_ctx: None,
            args: args.subst(ctx, subst),
            solution: self.solution.as_ref().map(|s| s.subst(ctx, subst)),
        }
    }
}

impl Print for Hole {
    fn print_prec<'a>(
        &'a self,
        cfg: &PrintCfg,
        alloc: &'a Alloc<'a>,
        prec: Precedence,
    ) -> Builder<'a> {
        match self.kind {
            MetaVarKind::MustSolve => {
                let mut doc = alloc.keyword(UNDERSCORE);

                if cfg.print_metavar_ids {
                    doc = doc.append(self.metavar.id.to_string());
                }

                if cfg.print_metavar_args {
                    doc = doc.append(print_hole_args(&self.args, cfg, alloc))
                }

                if cfg.print_metavar_solutions {
                    if let Some(solution) = &self.solution {
                        doc = doc
                            .append("<")
                            .append(solution.print_prec(cfg, alloc, prec))
                            .append(">")
                    }
                }

                doc
            }
            MetaVarKind::CanSolve => {
                let mut doc = alloc.keyword(QUESTION_MARK);

                if cfg.print_metavar_ids {
                    doc = doc.append(self.metavar.id.to_string());
                }

                if cfg.print_metavar_args {
                    doc = doc.append(print_hole_args(&self.args, cfg, alloc))
                }

                if cfg.print_metavar_solutions {
                    if let Some(solution) = &self.solution {
                        doc = doc
                            .append("<")
                            .append(solution.print_prec(cfg, alloc, prec))
                            .append(">")
                    }
                }

                doc
            }
            MetaVarKind::Inserted => {
                let mut doc = alloc.keyword(UNDERSCORE);

                if cfg.print_metavar_ids {
                    doc = doc.append(self.metavar.id.to_string());
                }

                if cfg.print_metavar_args {
                    doc = doc.append(print_hole_args(&self.args, cfg, alloc))
                }

                if cfg.print_metavar_solutions {
                    match &self.solution {
                        Some(solution) => {
                            doc = doc
                                .append("<")
                                .append(solution.print_prec(cfg, alloc, prec))
                                .append(">")
                        }
                        None => doc = doc.append("<Inserted>"),
                    }
                }

                doc
            }
        }
    }
}

fn print_hole_args<'a>(
    args: &'a [Vec<Binder<Box<Exp>>>],
    cfg: &PrintCfg,
    alloc: &'a Alloc<'a>,
) -> Builder<'a> {
    let groups = args.iter().map(|group| group.print(cfg, alloc).parens());
    let sep = alloc.text(COMMA).append(alloc.space());
    alloc.intersperse(groups, sep).append(alloc.text(COMMA).flat_alt(alloc.nil())).brackets()
}

impl Zonk for Hole {
    fn zonk(
        &mut self,
        meta_vars: &crate::HashMap<MetaVar, crate::MetaVarState>,
    ) -> Result<(), ZonkError> {
        // If the hole has already been solved, we zonk in the solution
        if let Some(solution) = &mut self.solution {
            solution.zonk(meta_vars)?;
            return Ok(());
        }
        // Otherwise, we check the metavars map
        match meta_vars.get(&self.metavar) {
            Some(crate::MetaVarState::Solved { ctx, solution }) => {
                // We assume that the metavars map is well-maintained, i.e. metavariables have already been zonked
                // in the solutions contained in the metavars map.
                // Assuming this invariant holds, we do not need to zonk here.
                // Unwrap is safe here because we are unwrapping an infallible result.
                let subst = Subst::from_binders(&self.args);
                self.solution = Some(solution.subst(&mut ctx.clone(), &subst));
            }
            Some(crate::MetaVarState::Unsolved { .. }) => {
                // Nothing to do, the hole remains unsolved
            }
            None => {
                return Err(ZonkError::UnboundMetaVar(self.metavar));
            }
        }

        Ok(())
    }
}

impl ContainsMetaVars for Hole {
    fn contains_metavars(&self) -> bool {
        let Hole { span: _, kind: _, metavar, inferred_type, inferred_ctx: _, args, solution } =
            self;

        inferred_type.contains_metavars()
            || args.contains_metavars()
            || solution.contains_metavars()
            || metavar.must_be_solved() && solution.is_none()
    }
}

impl Rename for Hole {
    fn rename_in_ctx(&mut self, ctx: &mut RenameCtx) {
        self.inferred_ctx = None;
        self.inferred_type.rename_in_ctx(ctx);
        self.args.rename_in_ctx(ctx);
    }
}

impl FreeVars for Hole {
    fn free_vars_mut(&self, ctx: &LevelCtx, cutoff: usize, fvs: &mut crate::HashSet<crate::Lvl>) {
        let Hole {
            span: _,
            kind: _,
            metavar: _,
            inferred_type: _,
            inferred_ctx: _,
            args,
            solution,
        } = self;

        args.free_vars_mut(ctx, cutoff, fvs);
        solution.free_vars_mut(ctx, cutoff, fvs);
    }
}
