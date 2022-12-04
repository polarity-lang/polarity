use pretty::DocAllocator;

use data::{Dec, No, Yes};

use super::types::*;

impl<'a, T: Print<'a>> Print<'a> for Dec<T> {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        match self {
            Yes(x) => x.print(cfg, alloc),
            No(_) => alloc.text("No".to_owned()),
        }
    }
}
