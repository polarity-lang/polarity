use pretty::DocAllocator;

use syntax::ast::Phase;
use syntax::ctx::TypeCtx;
use syntax::de_bruijn::ShiftCutoff;

use super::tokens::COMMA;
use super::Print;

impl<'a, P: Phase> Print<'a> for TypeCtx<P>
where
    P::Typ: ShiftCutoff,
{
    fn print(&'a self, cfg: &crate::PrintCfg, alloc: &'a crate::Alloc<'a>) -> crate::Builder<'a> {
        let iter = self.iter().map(|ctx| {
            alloc
                .intersperse(ctx.iter().map(|typ| typ.print(cfg, alloc)), alloc.text(COMMA))
                .brackets()
        });
        alloc.intersperse(iter, alloc.text(COMMA)).brackets()
    }
}