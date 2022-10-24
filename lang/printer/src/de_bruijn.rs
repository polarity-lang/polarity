use pretty::DocAllocator;

use syntax::de_bruijn::*;

use super::tokens::*;
use super::types::*;

impl<'a> Print<'a> for Idx {
    fn print(&'a self, _cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Idx { fst, snd } = self;
        alloc.text(AT).append(format!("{}", fst)).append(DOT).append(format!("{}", snd))
    }
}
