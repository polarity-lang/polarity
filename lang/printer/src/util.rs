use pretty::DocAllocator;

use super::types::*;

pub trait BracesExt<'a, D, A: 'a>
where
    D: ?Sized + DocAllocator<'a, A>,
{
    fn braces_from(self, cfg: &PrintCfg) -> pretty::DocBuilder<'a, D, A>;
}

pub trait BackslashExt<'a, A: 'a>: DocAllocator<'a, A> {
    fn backslash_from(&'a self, cfg: &PrintCfg) -> pretty::DocBuilder<'a, Self, A>;
}

pub trait IsNilExt<'a, D, A: 'a>
where
    D: ?Sized + DocAllocator<'a, A>,
{
    fn is_nil(&self) -> bool;
}

impl<'a, D, A> BracesExt<'a, D, A> for pretty::DocBuilder<'a, D, A>
where
    D: ?Sized + DocAllocator<'a, A>,
{
    fn braces_from(self, cfg: &PrintCfg) -> pretty::DocBuilder<'a, D, A> {
        let braces = if cfg.latex { ("\\{", "\\}") } else { ("{", "}") };
        self.enclose(braces.0, braces.1)
    }
}

impl<'a, A: 'a, T> BackslashExt<'a, A> for T
where
    T: DocAllocator<'a, A>,
{
    fn backslash_from(&'a self, cfg: &PrintCfg) -> pretty::DocBuilder<'a, Self, A> {
        let backlash = if cfg.latex { "\\textbackslash{}" } else { "\\" };
        self.text(backlash)
    }
}

impl<'a, D, A> IsNilExt<'a, D, A> for pretty::DocBuilder<'a, D, A>
where
    D: ?Sized + DocAllocator<'a, A>,
{
    fn is_nil(&self) -> bool {
        matches!(self.1, pretty::BuildDoc::Doc(pretty::Doc::Nil))
    }
}
