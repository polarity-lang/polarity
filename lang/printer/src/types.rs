use std::{error::Error, io, rc::Rc};

use pretty::{
    termcolor::{Ansi, WriteColor},
    DocAllocator,
};

use crate::{render, tokens::COMMA};

#[derive(Debug, Clone, Copy)]
pub enum Anno {
    Keyword,
    Ctor,
    Dtor,
    Type,
    Comment,
    Backslash,
    BraceOpen,
    BraceClose,
    Error,
}

pub type Alloc<'a> = pretty::Arena<'a, Anno>;
pub type Builder<'a> = pretty::DocBuilder<'a, Alloc<'a>, Anno>;

/// Operator precedences
pub type Precedence = u32;

/// We implement the `Print` trait for all types that we want to prettyprint.
/// It is sufficient to implement either the `print` or the `print_prec` function, depending
/// on whether you need information about operator precedences or not.
pub trait Print {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        Print::print_prec(self, cfg, alloc, 0)
    }

    /// Print with precedence information about the enclosing context.
    ///
    /// * `_prec` The precedence of the surrounding context.
    fn print_prec<'a>(
        &'a self,
        cfg: &PrintCfg,
        alloc: &'a Alloc<'a>,
        _prec: Precedence,
    ) -> Builder<'a> {
        Print::print(self, cfg, alloc)
    }

    fn print_io<W: io::Write>(&self, cfg: &PrintCfg, out: &mut W) -> io::Result<()> {
        let alloc = Alloc::new();
        let doc_builder = self.print(cfg, &alloc);
        doc_builder.1.render(cfg.width, out)
    }

    fn print_colored<W: WriteColor>(&self, cfg: &PrintCfg, out: &mut W) -> io::Result<()> {
        let alloc = Alloc::new();
        let doc_builder = self.print(cfg, &alloc);
        doc_builder.render_raw(cfg.width, &mut render::RenderTermcolor::new(out))
    }

    fn print_latex<W: io::Write>(&self, cfg: &PrintCfg, out: &mut W) -> io::Result<()> {
        let alloc = Alloc::new();
        let doc_builder = self.print(cfg, &alloc);
        doc_builder.render_raw(cfg.width, &mut render::RenderLatex::new(out))
    }

    fn print_to_string(&self, cfg: Option<&PrintCfg>) -> String {
        let mut buf = Vec::new();
        let def = PrintCfg::default();
        let cfg = cfg.unwrap_or(&def);
        self.print_io(cfg, &mut buf).expect("Failed to print to string");
        unsafe { String::from_utf8_unchecked(buf) }
    }

    fn print_to_colored_string(&self, cfg: Option<&PrintCfg>) -> String {
        let buf: Vec<u8> = Vec::new();
        let mut ansi = Ansi::new(buf);
        let def = PrintCfg::default();
        let cfg = cfg.unwrap_or(&def);
        self.print_colored(cfg, &mut ansi).expect("Failed to print to string");
        unsafe { String::from_utf8_unchecked(ansi.into_inner()) }
    }
}

impl Print for String {
    fn print<'a>(&'a self, _cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        alloc.text(self)
    }
}

impl<T: Print> Print for Vec<T> {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        print_comma_separated(self, cfg, alloc)
    }
}

pub fn print_comma_separated<'a, T: Print>(
    vec: &'a [T],
    cfg: &PrintCfg,
    alloc: &'a Alloc<'a>,
) -> Builder<'a> {
    if vec.is_empty() {
        alloc.nil()
    } else {
        let sep = alloc.text(COMMA).append(alloc.space());
        alloc.intersperse(vec.iter().map(|x| x.print(cfg, alloc)), sep)
    }
}

impl<T: Print> Print for Rc<T> {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        T::print(self, cfg, alloc)
    }

    fn print_prec<'a>(
        &'a self,
        cfg: &PrintCfg,
        alloc: &'a Alloc<'a>,
        prec: Precedence,
    ) -> Builder<'a> {
        T::print_prec(self, cfg, alloc, prec)
    }
}

impl<T: Print> Print for Option<T> {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        match self {
            Some(inner) => inner.print(cfg, alloc),
            None => alloc.nil(),
        }
    }
}

pub trait PrintInCtx {
    type Ctx;

    fn print_in_ctx<'a>(
        &'a self,
        cfg: &PrintCfg,
        ctx: &'a Self::Ctx,
        alloc: &'a Alloc<'a>,
    ) -> Builder<'a> {
        PrintInCtx::print_in_ctx_prec(self, cfg, ctx, alloc, 0)
    }

    /// Print with precedence information about the enclosing context.
    ///
    /// * `_prec` The precedence of the surrounding context.
    fn print_in_ctx_prec<'a>(
        &'a self,
        cfg: &PrintCfg,
        ctx: &'a Self::Ctx,
        alloc: &'a Alloc<'a>,
        _prec: Precedence,
    ) -> Builder<'a> {
        PrintInCtx::print_in_ctx(self, cfg, ctx, alloc)
    }

    fn print_to_string_in_ctx(&self, ctx: &Self::Ctx) -> String {
        let alloc = super::Alloc::new();
        let mut buf = Vec::new();
        {
            let cfg = PrintCfg::default();
            let doc_builder = self.print_in_ctx(&cfg, ctx, &alloc);
            doc_builder
                .1
                .render(super::DEFAULT_WIDTH, &mut buf)
                .expect("Failed to print to string");
            drop(doc_builder)
        }
        unsafe { String::from_utf8_unchecked(buf) }
    }
}

impl<T: Print> Print for &T {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        T::print(self, cfg, alloc)
    }

    fn print_prec<'a>(
        &'a self,
        cfg: &PrintCfg,
        alloc: &'a Alloc<'a>,
        prec: Precedence,
    ) -> Builder<'a> {
        T::print_prec(self, cfg, alloc, prec)
    }
}

impl<T: Print, E: Error> Print for Result<T, E> {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        match self {
            Ok(x) => x.print(cfg, alloc),
            Err(err) => alloc.text(err.to_string()).annotate(Anno::Error),
        }
    }

    fn print_prec<'a>(
        &'a self,
        cfg: &PrintCfg,
        alloc: &'a Alloc<'a>,
        prec: Precedence,
    ) -> Builder<'a> {
        match self {
            Ok(x) => x.print_prec(cfg, alloc, prec),
            Err(err) => alloc.text(err.to_string()).annotate(Anno::Error),
        }
    }
}

#[derive(Clone)]
pub struct PrintCfg {
    /// The width of the output terminal/device. Width is used for
    /// the insertion of linebreaks.
    pub width: usize,
    /// Whether to escape braces and backslashes
    pub latex: bool,
    /// Whether to omit the empty line between toplevel declarations.
    pub omit_decl_sep: bool,
    /// Whether to print the De-Bruijn representation of variables
    pub de_bruijn: bool,
    /// How many spaces of indentation are used
    pub indent: isize,
    /// Whether to print the syntactic sugar "\x. body".
    pub print_lambda_sugar: bool,
    /// Whether to print the syntactic sugar "a -> b".
    pub print_function_sugar: bool,
    /// Whether to print the ids of metavariables
    pub print_metavar_ids: bool,
}

impl Default for PrintCfg {
    fn default() -> Self {
        Self {
            width: crate::DEFAULT_WIDTH,
            latex: false,
            omit_decl_sep: false,
            de_bruijn: false,
            indent: 4,
            print_lambda_sugar: true,
            print_function_sugar: true,
            print_metavar_ids: false,
        }
    }
}
