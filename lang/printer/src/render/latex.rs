use std::io;

use crate::types::*;

pub struct RenderLatex<W> {
    anno_stack: Vec<Anno>,
    upstream: W,
}

impl<W> RenderLatex<W> {
    pub fn new(upstream: W) -> RenderLatex<W> {
        RenderLatex { anno_stack: Vec::new(), upstream }
    }
}

impl<W> pretty::Render for RenderLatex<W>
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

impl<W> pretty::RenderAnnotated<'_, Anno> for RenderLatex<W>
where
    W: io::Write,
{
    fn push_annotation(&mut self, anno: &Anno) -> Result<(), Self::Error> {
        self.anno_stack.push(*anno);
        let out = match anno {
            Anno::Keyword => "\\polKw{",
            Anno::Ctor => "\\polCtor{",
            Anno::Dtor => "\\polDtor{",
            Anno::Type => "\\polType{",
            Anno::Identifier => "\\polCtor{", // TODO: Change this?
            Anno::Comment => "\\polComment{",
            // Produce a backslash
            Anno::Backslash => "\\polBackslash{",
            // Escape an opening brace that follows immediately
            Anno::BraceOpen => "\\",
            // Escape a closing brace that follows immediately
            Anno::BraceClose => "\\",
            Anno::Error => "\\textcolor{polRed}{",
        };
        self.upstream.write_all(out.as_bytes())
    }

    fn pop_annotation(&mut self) -> Result<(), Self::Error> {
        let res = match self.anno_stack.last() {
            Some(Anno::BraceOpen) | Some(Anno::BraceClose) => Ok(()),
            _ => self.upstream.write_all("}".as_bytes()),
        };
        self.anno_stack.pop();
        res
    }
}
