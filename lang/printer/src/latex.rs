use std::io;

use pretty::termcolor::{Color, WriteColor};

pub struct LatexWriter<'a, W: io::Write> {
    inner: &'a mut W,
}

impl<'a, W: io::Write> LatexWriter<'a, W> {
    pub fn new(inner: &'a mut W) -> Self {
        Self { inner }
    }
}

impl<'a, W: io::Write> io::Write for LatexWriter<'a, W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

impl<'a, W: io::Write> WriteColor for LatexWriter<'a, W> {
    fn supports_color(&self) -> bool {
        true
    }

    fn set_color(&mut self, spec: &pretty::termcolor::ColorSpec) -> std::io::Result<()> {
        let cmd = spec
            .fg()
            .and_then(termcolor_to_latex)
            .map(|xcolor| xcolor.text_color())
            .map(|cmd| format!("{cmd}{{"));

        match cmd {
            Some(s) => self.inner.write(s.as_bytes()).map(|_| ()),
            None => self.inner.write("{".as_bytes()).map(|_| ()),
        }
    }

    fn reset(&mut self) -> std::io::Result<()> {
        self.inner.write("}".as_bytes()).map(|_| ())
    }
}

pub enum LatexColor {
    Known(&'static str),
}

impl LatexColor {
    fn text_color(&self) -> String {
        match self {
            LatexColor::Known(s) => format!("\\textcolor{{{s}}}"),
        }
    }
}

fn termcolor_to_latex(color: &Color) -> Option<LatexColor> {
    use LatexColor::*;
    let out = match color {
        Color::Black => Known("black"),
        Color::Blue => Known("blue"),
        Color::Green => Known("darkgreen"),
        Color::Red => Known("red"),
        Color::Cyan => Known("cyan"),
        Color::Magenta => Known("magenta"),
        Color::Yellow => Known("yellow"),
        Color::White => Known("white"),
        _ => return None,
    };
    Some(out)
}
