use pretty::DocAllocator;

use syntax::common::*;

use super::tokens::*;
use super::types::*;

impl Print for Idx {
    fn print<'a>(&'a self, _cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Idx { fst, snd } = self;
        alloc.text(AT).append(format!("{fst}")).append(DOT).append(format!("{snd}"))
    }
}
