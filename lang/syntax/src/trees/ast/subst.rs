use std::rc::Rc;

use crate::ast::Variable;
use crate::common::*;
use crate::ctx::*;

use super::*;

impl<T: Substitutable> SwapWithCtx for T {
    fn swap_with_ctx(
        &self,
        ctx: &mut LevelCtx,
        fst1: usize,
        fst2: usize,
    ) -> <T as Substitutable>::Result {
        self.subst(ctx, &SwapSubst { fst1, fst2 })
    }
}

#[derive(Clone)]
struct SwapSubst {
    fst1: usize,
    fst2: usize,
}

impl Shift for SwapSubst {
    fn shift_in_range<R: ShiftRange>(&self, _range: R, _by: (isize, isize)) -> Self {
        // Since SwapSubst works with levels, it is shift-invariant
        self.clone()
    }
}

impl Substitution for SwapSubst {
    fn get_subst(&self, ctx: &LevelCtx, lvl: Lvl) -> Option<Rc<Exp>> {
        let new_lvl = if lvl.fst == self.fst1 {
            Some(Lvl { fst: self.fst2, snd: lvl.snd })
        } else if lvl.fst == self.fst2 {
            Some(Lvl { fst: self.fst1, snd: lvl.snd })
        } else {
            None
        };

        let new_ctx = ctx.swap(self.fst1, self.fst2);

        new_lvl.map(|new_lvl| {
            Rc::new(Exp::Variable(Variable {
                span: None,
                idx: new_ctx.lvl_to_idx(new_lvl),
                name: "".to_owned(),
                inferred_type: None,
            }))
        })
    }
}
