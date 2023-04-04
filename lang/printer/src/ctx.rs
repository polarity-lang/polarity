use pretty::DocAllocator;

use syntax::ctx::values;

use super::tokens::COMMA;
use super::Print;

impl<'a> Print<'a> for values::TypeCtx {
    fn print(&'a self, cfg: &crate::PrintCfg, alloc: &'a crate::Alloc<'a>) -> crate::Builder<'a> {
        let iter = self.iter().map(|ctx| {
            alloc
                .intersperse(ctx.iter().map(|typ| typ.print(cfg, alloc)), alloc.text(COMMA))
                .brackets()
        });
        alloc.intersperse(iter, alloc.text(COMMA)).brackets()
    }
}
