use std::error::Error;

use pretty::termcolor::{Color, ColorSpec};
use pretty::DocAllocator;

use crate::theme::ColorExt;

pub type Alloc<'a> = pretty::Arena<'a, ColorSpec>;
pub type Builder<'a> = pretty::DocBuilder<'a, Alloc<'a>, ColorSpec>;

pub trait Print<'a> {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a>;
}

pub trait PrintInCtx<'a> {
    type Ctx;

    fn print_in_ctx(
        &'a self,
        cfg: &PrintCfg,
        ctx: &'a Self::Ctx,
        alloc: &'a Alloc<'a>,
    ) -> Builder<'a>;
}

impl<'a, T: Print<'a>> Print<'a> for &T {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        T::print(self, cfg, alloc)
    }
}

impl<'a, T: Print<'a>, E: Error> Print<'a> for Result<T, E> {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        match self {
            Ok(x) => x.print(cfg, alloc),
            Err(err) => alloc.text(err.to_string()).annotate(Color::Red.spec()),
        }
    }
}

pub struct PrintCfg {
    pub width: usize,
    pub braces: (&'static str, &'static str),
    /// Whether to omit the empty line between toplevel declarations.
    pub omit_decl_sep: bool,
    /// Whether to print the De-Bruijn representation of variables
    pub de_bruijn: bool,
    /// How many spaces of indentation are used
    pub indent: isize,
    /// Whether to print the syntactic sugar "\x. body".
    pub print_lambda_sugar: bool,
}

impl Default for PrintCfg {
    fn default() -> Self {
        Self {
            width: crate::DEFAULT_WIDTH,
            braces: ("{", "}"),
            omit_decl_sep: false,
            de_bruijn: false,
            indent: 4,
            print_lambda_sugar: true,
        }
    }
}
