use std::io;

pub use pretty::termcolor;
pub use pretty::termcolor::Color;
pub use pretty::termcolor::ColorChoice;
pub use pretty::termcolor::ColorSpec;
pub use pretty::termcolor::StandardStream;
pub use pretty::termcolor::WriteColor;
pub use pretty::DocAllocator;

mod ctx;
mod de_bruijn;
pub mod fragments;
mod generic;
mod print_to_string;
mod render;
pub mod theme;
pub mod tokens;
pub mod types;
pub mod util;

pub use print_to_string::*;
pub use types::*;

pub const DEFAULT_WIDTH: usize = 100;

pub trait PrintExt {
    fn print_io<W: io::Write>(&self, cfg: &PrintCfg, out: &mut W) -> io::Result<()>;
    fn print_colored<W: WriteColor>(&self, cfg: &PrintCfg, out: &mut W) -> io::Result<()>;
    fn print_latex<W: io::Write>(&self, cfg: &PrintCfg, out: &mut W) -> io::Result<()>;
}

impl<T: for<'a> Print<'a>> PrintExt for T {
    fn print_io<W: io::Write>(&self, cfg: &PrintCfg, out: &mut W) -> io::Result<()> {
        let alloc = Alloc::new();
        let doc_builder = T::print(self, cfg, &alloc);
        doc_builder.1.render(cfg.width, out)
    }

    fn print_colored<W: WriteColor>(&self, cfg: &PrintCfg, out: &mut W) -> io::Result<()> {
        let alloc = Alloc::new();
        let doc_builder = T::print(self, cfg, &alloc);
        doc_builder.render_raw(cfg.width, &mut render::RenderTermcolor::new(out))
    }

    fn print_latex<W: io::Write>(&self, cfg: &PrintCfg, out: &mut W) -> io::Result<()> {
        let alloc = Alloc::new();
        let doc_builder = T::print(self, cfg, &alloc);
        doc_builder.render_raw(cfg.width, &mut render::RenderLatex::new(out))
    }
}
