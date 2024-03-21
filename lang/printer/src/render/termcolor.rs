use std::io;

use pretty::termcolor::{Color, ColorSpec};

use crate::types::*;
use crate::WriteColor;

const KEYWORD: Color = Color::Magenta;
const CTOR: Color = Color::Blue;
const DTOR: Color = Color::Green;
const TYPE: Color = Color::Red;
const IDENTIFIER: Color = Color::Blue; // TODO: Change this?
const COMMENT: Color = Color::Cyan;
const ERROR: Color = Color::Red;

pub struct RenderTermcolor<W> {
    anno_stack: Vec<Anno>,
    upstream: W,
}

impl<W> RenderTermcolor<W> {
    pub fn new(upstream: W) -> RenderTermcolor<W> {
        RenderTermcolor { anno_stack: Vec::new(), upstream }
    }
}

impl<W> pretty::Render for RenderTermcolor<W>
where
    W: io::Write,
{
    type Error = io::Error;

    fn write_str(&mut self, s: &str) -> io::Result<usize> {
        self.upstream.write(s.as_bytes())
    }

    fn write_str_all(&mut self, s: &str) -> io::Result<()> {
        self.upstream.write_all(s.as_bytes())
    }

    fn fail_doc(&self) -> Self::Error {
        io::Error::new(io::ErrorKind::Other, "Document failed to render")
    }
}

impl<W> pretty::RenderAnnotated<'_, Anno> for RenderTermcolor<W>
where
    W: WriteColor,
{
    fn push_annotation(&mut self, anno: &Anno) -> Result<(), Self::Error> {
        self.anno_stack.push(*anno);
        self.upstream.set_color(&anno.color_spec())
    }

    fn pop_annotation(&mut self) -> Result<(), Self::Error> {
        self.anno_stack.pop();
        match self.anno_stack.last() {
            Some(previous) => self.upstream.set_color(&previous.color_spec()),
            None => self.upstream.reset(),
        }
    }
}

impl Anno {
    fn color_spec(&self) -> ColorSpec {
        match self {
            Anno::Keyword => KEYWORD.spec(),
            Anno::Ctor => CTOR.spec(),
            Anno::Dtor => DTOR.spec(),
            Anno::Type => TYPE.spec(),
            Anno::Identifier => IDENTIFIER.spec(),
            Anno::Comment => COMMENT.spec(),
            Anno::Backslash => KEYWORD.spec(),
            Anno::BraceOpen => Default::default(),
            Anno::BraceClose => Default::default(),
            Anno::Error => ERROR.spec(),
        }
    }
}

pub trait ColorExt {
    fn spec(self) -> ColorSpec;
}

impl ColorExt for Color {
    fn spec(self) -> ColorSpec {
        ColorSpec::new().set_fg(Some(self)).clone()
    }
}
