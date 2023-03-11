use pretty::DocAllocator;

use super::types::*;

pub trait BracesExt<'a, D, A: 'a>
where
    D: ?Sized + DocAllocator<'a, A>,
{
    fn braces_from(self, cfg: &PrintCfg) -> pretty::DocBuilder<'a, D, A>;
}

impl<'a, D, A> BracesExt<'a, D, A> for pretty::DocBuilder<'a, D, A>
where
    D: ?Sized + DocAllocator<'a, A>,
{
    fn braces_from(self, cfg: &PrintCfg) -> pretty::DocBuilder<'a, D, A> {
        self.enclose(cfg.braces.0, cfg.braces.1)
    }
}
