use pretty::DocAllocator;

use syntax::de_bruijn::*;
use syntax::var::*;

use super::tokens::*;
use super::types::*;

impl<'a> Print<'a> for Var {
    fn print(&'a self, alloc: &'a Alloc<'a>) -> Builder<'a> {
        match self {
            Var::Bound(Idx { fst, snd }) => {
                alloc.text(AT).append(format!("{}", fst)).append(DOT).append(format!("{}", snd))
            }
            Var::Free(name) => alloc.text(name),
        }
    }
}
