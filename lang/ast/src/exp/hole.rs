use derivative::Derivative;
use miette_util::codespan::Span;
use pretty::DocAllocator;
use printer::{
    theme::ThemeExt,
    tokens::{QUESTION_MARK, UNDERSCORE},
    Alloc, Builder, Precedence, Print, PrintCfg,
};

use crate::{
    ctx::{values::TypeCtx, LevelCtx},
    ContainsMetaVars, HasSpan, HasType, Occurs, Shift, ShiftRange, SubstUnderCtx, Substitutable,
    Substitution, Zonk, ZonkError,
};

use super::{Exp, Lvl, MetaVar, MetaVarKind};

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
    pub args: Vec<Vec<Box<Exp>>>,
    /// The solution found by unification. It is propagated during zonking.
    pub solution: Option<Box<Exp>>,
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
    fn occurs(&self, _ctx: &mut LevelCtx, _lvl: Lvl) -> bool {
        false
    }
}

impl HasType for Hole {
    fn typ(&self) -> Option<Box<Exp>> {
        self.inferred_type.clone()
    }
}

impl Substitutable for Hole {
    type Result = Hole;

    fn subst<S: Substitution>(&self, ctx: &mut LevelCtx, by: &S) -> Self::Result {
        let Hole { span, kind, metavar, args, .. } = self;
        Hole {
            span: *span,
            kind: *kind,
            metavar: *metavar,
            inferred_type: None,
            inferred_ctx: None,
            args: args.subst(ctx, by),
            solution: self.solution.subst(ctx, by),
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

                if let Some(solution) = &self.solution {
                    doc = doc.append("<").append(solution.print_prec(cfg, alloc, prec)).append(">")
                }

                doc
            }
            MetaVarKind::CanSolve => {
                let mut doc = alloc.keyword(QUESTION_MARK);

                if cfg.print_metavar_ids {
                    doc = doc.append(self.metavar.id.to_string());
                }

                if let Some(solution) = &self.solution {
                    doc = doc.append("<").append(solution.print_prec(cfg, alloc, prec)).append(">")
                }

                doc
            }
            MetaVarKind::Inserted => {
                let mut doc = alloc.nil();

                if cfg.print_metavar_ids {
                    doc = doc.append(self.metavar.id.to_string());
                }

                match &self.solution {
                    Some(solution) => {
                        doc = doc
                            .append("<")
                            .append(solution.print_prec(cfg, alloc, prec))
                            .append(">")
                    }
                    None => doc = doc.append("<Inserted>"),
                }

                doc
            }
        }
    }
}

impl Zonk for Hole {
    fn zonk(
        &mut self,
        meta_vars: &crate::HashMap<MetaVar, crate::MetaVarState>,
    ) -> Result<(), ZonkError> {
        match meta_vars.get(&self.metavar) {
            Some(crate::MetaVarState::Solved { ctx, solution }) => {
                self.solution = Some(solution.subst_under_ctx(ctx.clone(), &self.args));
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
