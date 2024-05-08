use pretty::DocAllocator;

use super::types::*;
use syntax::ast::{Decl, Module};

pub struct Items {
    pub items: Vec<Decl>,
}

impl<'a> PrintInCtx<'a> for Items {
    type Ctx = Module;

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
