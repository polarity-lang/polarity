use std::rc::Rc;

use crate::ast::exp::Exp;
use crate::ast::{Idx, Lvl};
use crate::ctx::LevelCtx;

pub trait Occurs {
    fn occurs(&self, ctx: &mut LevelCtx, lvl: Lvl) -> bool;
}

pub fn occurs_in(ctx: &mut LevelCtx, the_idx: Idx, in_exp: &Rc<Exp>) -> bool {
    let lvl = ctx.idx_to_lvl(the_idx);
    in_exp.occurs(ctx, lvl)
}

impl<T: Occurs> Occurs for Rc<T> {
    fn occurs(&self, ctx: &mut LevelCtx, lvl: Lvl) -> bool {
        T::occurs(self, ctx, lvl)
    }
}

impl<T: Occurs> Occurs for Vec<T> {
    fn occurs(&self, ctx: &mut LevelCtx, lvl: Lvl) -> bool {
        self.iter().any(|x| x.occurs(ctx, lvl))
    }
}

impl<T: Occurs> Occurs for Option<T> {
    fn occurs(&self, ctx: &mut LevelCtx, lvl: Lvl) -> bool {
        self.as_ref().map(|inner| inner.occurs(ctx, lvl)).unwrap_or_default()
    }
}
