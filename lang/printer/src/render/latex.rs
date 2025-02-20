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
        if matches!(self.anno_stack.last(), Some(Anno::Reference(_, _))) {
            Ok(0)
        } else {
            self.upstream.write(s.as_bytes())
        }
    }

    fn write_str_all(&mut self, s: &str) -> io::Result<()> {
        if matches!(self.anno_stack.last(), Some(Anno::Reference(_, _))) {
            Ok(())
        } else {
            self.upstream.write_all(s.as_bytes())
        }
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
        self.anno_stack.push(anno.clone());
        let out = match anno {
            Anno::Keyword => r"\polKw{",
            Anno::Ctor => r"\polCtor{",
            Anno::Dtor => r"\polDtor{",
            Anno::Type => r"\polType{",
            Anno::Comment => r"\polComment{",
            // Produce a backslash
            Anno::Backslash => r"\polBackslash{",
            // Escape an opening brace that follows immediately
            Anno::BraceOpen => r"\",
            // Escape a closing brace that follows immediately
            Anno::BraceClose => r"\",
            Anno::Error => r"\textcolor{polRed}{",
            Anno::Reference(_, _) => "",
        };
        self.upstream.write_all(out.as_bytes())
    }

    fn pop_annotation(&mut self) -> Result<(), Self::Error> {
        let res = match self.anno_stack.last() {
            Some(Anno::BraceOpen) | Some(Anno::BraceClose) | Some(Anno::Reference(_, _)) => Ok(()),
            _ => self.upstream.write_all("}".as_bytes()),
        };
        self.anno_stack.pop();
        res
    }
}
