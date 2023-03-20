use pretty::DocAllocator;

use super::types::*;

pub trait BracesExt<'a, D>
where
    D: ?Sized + DocAllocator<'a, Anno>,
{
    fn braces_anno(self) -> pretty::DocBuilder<'a, D, Anno>;
}

pub trait BackslashExt<'a>: DocAllocator<'a, Anno> {
    fn backslash_anno(&'a self, cfg: &PrintCfg) -> pretty::DocBuilder<'a, Self, Anno>;
}

pub trait IsNilExt<'a, D, A: 'a>
where
    D: ?Sized + DocAllocator<'a, A>,
{
    fn is_nil(&self) -> bool;
}

impl<'a, D> BracesExt<'a, D> for pretty::DocBuilder<'a, D, Anno>
where
    D: ?Sized + DocAllocator<'a, Anno>,
{
    fn braces_anno(self) -> pretty::DocBuilder<'a, D, Anno> {
        let l = self.0.text("{".to_owned()).annotate(Anno::BraceOpen);
        let r = self.0.text("}".to_owned()).annotate(Anno::BraceClose);
        self.enclose(l, r)
    }
}

impl<'a, T> BackslashExt<'a> for T
where
    T: DocAllocator<'a, Anno>,
{
    fn backslash_anno(&'a self, cfg: &PrintCfg) -> pretty::DocBuilder<'a, Self, Anno> {
        let backlash = if cfg.latex { " " } else { "\\" };
        self.text(backlash).annotate(Anno::Backslash)
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
