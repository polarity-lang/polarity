use std::rc::Rc;

use crate::common::*;
use crate::ctx::*;

use super::def::*;

pub fn occurs_in<P: Phase>(ctx: &mut LevelCtx, the_idx: Idx, in_exp: &Rc<Exp<P>>) -> bool {
    let lvl = ctx.idx_to_lvl(the_idx);
    in_exp.occurs(ctx, lvl)
}

trait Occurs {
    fn occurs(&self, ctx: &mut LevelCtx, lvl: Lvl) -> bool;
}

impl<P: Phase> Occurs for Exp<P> {
    fn occurs(&self, ctx: &mut LevelCtx, lvl: Lvl) -> bool {
        match self {
            Exp::Var { idx, .. } => ctx.idx_to_lvl(*idx) == lvl,
            Exp::TypCtor { args, .. } => args.iter().any(|arg| arg.occurs(ctx, lvl)),
            Exp::Ctor { args, .. } => args.iter().any(|arg| arg.occurs(ctx, lvl)),
            Exp::Dtor { exp, args, .. } => {
                exp.occurs(ctx, lvl) || args.iter().any(|arg| arg.occurs(ctx, lvl))
            }
            Exp::Anno { exp, typ, .. } => exp.occurs(ctx, lvl) || typ.occurs(ctx, lvl),
            Exp::Type { .. } => false,
            Exp::Match { on_exp, body, .. } => on_exp.occurs(ctx, lvl) || body.occurs(ctx, lvl),
            Exp::Comatch { body, .. } => body.occurs(ctx, lvl),
            Exp::Hole { info: _ } => todo!(),
        }
    }
}

impl<P: Phase> Occurs for Match<P> {
    fn occurs(&self, ctx: &mut LevelCtx, lvl: Lvl) -> bool {
        let Match { cases, .. } = self;
        cases.occurs(ctx, lvl)
    }
}

impl<P: Phase> Occurs for Comatch<P> {
    fn occurs(&self, ctx: &mut LevelCtx, lvl: Lvl) -> bool {
        let Comatch { cases, .. } = self;
        cases.occurs(ctx, lvl)
    }
}

impl<P: Phase> Occurs for Case<P> {
    fn occurs(&self, ctx: &mut LevelCtx, lvl: Lvl) -> bool {
        let Case { args, body, .. } = self;
        ctx.bind_iter(args.params.iter().map(|_| ()), |ctx| body.occurs(ctx, lvl))
    }
}

impl<P: Phase> Occurs for Cocase<P> {
    fn occurs(&self, ctx: &mut LevelCtx, lvl: Lvl) -> bool {
        let Cocase { params: args, body, .. } = self;
        ctx.bind_iter(args.params.iter().map(|_| ()), |ctx| body.occurs(ctx, lvl))
    }
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
