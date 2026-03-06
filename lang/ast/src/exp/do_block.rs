use derivative::Derivative;

use polarity_lang_miette_util::codespan::Span;
use polarity_lang_printer::Print;

use super::{Exp, VarBind};
use crate::{
    ContainsMetaVars, FreeVars, HasSpan, HasType, Occurs, Shift, Subst, Substitutable, Zonk,
    ctx::LevelCtx, rename::Rename,
};

/// Do block:
/// ```text
/// do {
///   let x <- foo();
///   y <- bar();
///   x
/// }
/// ```
#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct DoBlock {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub span: Span,

    /// The lets and binds of the do block.
    pub bindings: Vec<DoBinding>,

    /// The final return expression of the do block.
    pub return_exp: Box<Exp>,

    /// Type of the do block inferred during elaboration.
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub inferred_type: Option<Box<Exp>>,
}

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub enum DoBinding {
    Bind {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        span: Span,
        var: VarBind,
        exp: Box<Exp>,
    },
    Let {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        span: Span,
        var: VarBind,
        typ: Option<Box<Exp>>,
        exp: Box<Exp>,
    },
}

impl HasSpan for DoBlock {
    fn span(&self) -> Option<Span> {
        Some(self.span)
    }
}

impl Shift for DoBlock {
    fn shift_in_range<R: crate::ShiftRange>(&mut self, range: &R, by: (isize, isize)) {
        let DoBlock { span: _, bindings, return_exp, inferred_type } = self;
        todo!()
    }
}

impl Occurs for DoBlock {
    fn occurs<F>(&self, ctx: &mut crate::ctx::LevelCtx, f: &F) -> bool
    where
        F: Fn(&crate::ctx::LevelCtx, &Exp) -> bool,
    {
        let DoBlock { span, bindings, return_exp, inferred_type } = self;
        todo!()
    }
}

impl HasType for DoBlock {
    fn typ(&self) -> Option<Box<Exp>> {
        self.inferred_type.clone()
    }
}

impl Substitutable for DoBlock {
    type Target = DoBlock;

    fn subst(&self, ctx: &mut LevelCtx, subst: &Subst) -> Self::Target {
        let DoBlock { span, bindings, return_exp, inferred_type } = self;
        todo!()
    }
}

impl Print for DoBlock {
    fn print_prec<'a>(
        &'a self,
        cfg: &polarity_lang_printer::PrintCfg,
        alloc: &'a polarity_lang_printer::Alloc<'a>,
        _prec: polarity_lang_printer::Precedence,
    ) -> polarity_lang_printer::Builder<'a> {
        let DoBlock { span, bindings, return_exp, inferred_type } = self;
        todo!()
    }
}

impl Zonk for DoBlock {
    fn zonk(
        &mut self,
        meta_vars: &crate::HashMap<crate::MetaVar, crate::MetaVarState>,
    ) -> Result<(), crate::ZonkError> {
        let DoBlock { span, bindings, return_exp, inferred_type } = self;
        todo!()
    }
}

impl ContainsMetaVars for DoBlock {
    fn contains_metavars(&self) -> bool {
        let DoBlock { span, bindings, return_exp, inferred_type } = self;
        todo!()
    }
}

impl Rename for DoBlock {
    fn rename_in_ctx(&mut self, ctx: &mut crate::rename::RenameCtx) {
        let DoBlock { span, bindings, return_exp, inferred_type } = self;
        todo!()
    }
}

impl From<DoBlock> for Exp {
    fn from(val: DoBlock) -> Self {
        Exp::DoBlock(val)
    }
}

impl FreeVars for DoBlock {
    fn free_vars_mut(&self, ctx: &LevelCtx, cutoff: usize, fvs: &mut crate::HashSet<crate::Lvl>) {
        let DoBlock { span, bindings, return_exp, inferred_type } = self;
        todo!()
    }
}
