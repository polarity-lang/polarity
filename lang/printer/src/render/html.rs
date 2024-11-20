use std::io;

use crate::types::*;

pub struct RenderHtml<W> {
    anno_stack: Vec<Anno>,
    upstream: W,
}

impl<W> RenderHtml<W> {
    pub fn new(upstream: W) -> RenderHtml<W> {
        RenderHtml { anno_stack: Vec::new(), upstream }
    }
}

impl<W> pretty::Render for RenderHtml<W>
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

impl<W> pretty::RenderAnnotated<'_, Anno> for RenderHtml<W>
where
    W: io::Write,
{
    fn push_annotation(&mut self, anno: &Anno) -> Result<(), Self::Error> {
        self.anno_stack.push(*anno);
        let out = match anno {
            Anno::Keyword => "<span class=\"keyword\">",
            Anno::Ctor => "<span class=\"ctor\">",
            Anno::Dtor => "<span class=\"dtor\">",
            Anno::Type => "<span class=\"type\">",
            Anno::Comment => "<span class=\"comment\">",
            Anno::Backslash => "",
            Anno::BraceOpen => "",
            Anno::BraceClose => "",
            Anno::Error => "<span class=\"error\">",
        };
        self.upstream.write_all(out.as_bytes())
    }

    fn pop_annotation(&mut self) -> Result<(), Self::Error> {
        let res = match self.anno_stack.last() {
            Some(Anno::Backslash) | Some(Anno::BraceOpen) | Some(Anno::BraceClose) => Ok(()),
            _ => self.upstream.write_all("</span>".as_bytes()),
        };
        self.anno_stack.pop();
        res
    }
}
