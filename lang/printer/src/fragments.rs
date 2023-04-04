use pretty::DocAllocator;

use syntax::ust;

use super::types::*;

pub struct Items {
    pub items: Vec<ust::Decl>,
}

impl<'a> PrintInCtx<'a> for Items {
    type Ctx = ust::Decls;

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
