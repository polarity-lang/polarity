use std::rc::Rc;

use super::TelescopeInst;
use crate::common::{Idx, Leveled, Lvl};
use crate::ctx::LevelCtx;

use super::exp::Exp;

// Occurs
//
//

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

// Instantiate
//
//

pub trait Instantiate {
    fn instantiate(&self) -> TelescopeInst;
}

// HasTypeInfo
//
//

pub trait HasTypeInfo {
    fn typ(&self) -> Option<Rc<Exp>>;
}

// ForgetTST
//
//

pub trait ForgetTST {
    fn forget_tst(&self) -> Self;
}

impl<T: ForgetTST> ForgetTST for Rc<T> {
    fn forget_tst(&self) -> Self {
        Rc::new(T::forget_tst(self))
    }
}

impl<T: ForgetTST> ForgetTST for Vec<T> {
    fn forget_tst(&self) -> Self {
        self.iter().map(ForgetTST::forget_tst).collect()
    }
}
