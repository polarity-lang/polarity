use std::io;

pub use pretty::termcolor::ColorChoice;
pub use pretty::termcolor::StandardStream;
pub use pretty::termcolor::WriteColor;

mod ast;
mod common;
mod theme;
mod tokens;
mod types;

pub use types::*;

pub const DEFAULT_WIDTH: usize = 100;

pub trait PrintExt<'a, T: Print<'a>> {
    fn print<W: io::Write>(&'a self, x: &'a T, width: usize, out: &mut W) -> io::Result<()>;
    fn print_colored<W: WriteColor>(&'a self, x: &'a T, width: usize, out: W) -> io::Result<()>;
}

impl<'a, T: Print<'a>> PrintExt<'a, T> for Alloc<'a> {
    fn print<W: io::Write>(&'a self, x: &'a T, width: usize, out: &mut W) -> io::Result<()> {
        let doc_builder = x.print(self);
        doc_builder.1.render(width, out)
    }

    fn print_colored<W: WriteColor>(&'a self, x: &'a T, width: usize, out: W) -> io::Result<()> {
        let doc_builder = x.print(self);
        doc_builder.1.render_colored(width, out)
    }
}
