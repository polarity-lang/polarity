use std::error::Error;

use pretty::termcolor::{Color, ColorSpec};
use pretty::DocAllocator;

use crate::theme::ColorExt;

pub type Alloc<'a> = pretty::Arena<'a, ColorSpec>;
pub type Builder<'a> = pretty::DocBuilder<'a, Alloc<'a>, ColorSpec>;

pub trait Print<'a> {
    fn print(&'a self, alloc: &'a Alloc<'a>) -> Builder<'a>;
}

pub trait PrintInCtx<'a> {
    type Ctx;

    fn print_in_ctx(&'a self, ctx: &'a Self::Ctx, alloc: &'a Alloc<'a>) -> Builder<'a>;
}

impl<'a, T: Print<'a>> Print<'a> for &T {
    fn print(&'a self, alloc: &'a Alloc<'a>) -> Builder<'a> {
        T::print(self, alloc)
    }
}

impl<'a, T: Print<'a>, E: Error> Print<'a> for Result<T, E> {
    fn print(&'a self, alloc: &'a Alloc<'a>) -> Builder<'a> {
        match self {
            Ok(x) => x.print(alloc),
            Err(err) => alloc.text(err.to_string()).annotate(Color::Red.spec()),
        }
    }
}
