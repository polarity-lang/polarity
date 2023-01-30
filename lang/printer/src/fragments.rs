use pretty::DocAllocator;

use syntax::ast::*;
use syntax::common::*;

use super::types::*;

pub struct Items<P: Phase> {
    pub items: Vec<Decl<P>>,
}

impl<'a, P: Phase> PrintInCtx<'a> for Items<P>
where
    P::Typ: ShiftInRange,
{
    type Ctx = Decls<P>;

    fn print_in_ctx(
        &'a self,
        cfg: &PrintCfg,
        ctx: &'a Self::Ctx,
        alloc: &'a Alloc<'a>,
    ) -> Builder<'a> {
        let Items { items } = self;

        let sep = if cfg.omit_decl_sep { alloc.line() } else { alloc.line().append(alloc.line()) };
        alloc.intersperse(items.iter().map(|item| item.print_in_ctx(cfg, ctx, alloc)), sep)
    }
}
