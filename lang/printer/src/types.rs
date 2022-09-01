use pretty::termcolor::ColorSpec;

pub type Alloc<'a> = pretty::Arena<'a, ColorSpec>;
pub type Builder<'a> = pretty::DocBuilder<'a, Alloc<'a>, ColorSpec>;

pub trait Print<'a> {
    fn print(&'a self, alloc: &'a Alloc<'a>) -> Builder<'a>;
}

pub trait PrintInCtx<'a> {
    type Ctx;

    fn print_in_ctx(&'a self, ctx: &'a Self::Ctx, alloc: &'a Alloc<'a>) -> Builder<'a>;
}
