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
        self.write_str_all(s).map(|()| s.len())
    }

    fn write_str_all(&mut self, s: &str) -> io::Result<()> {
        if matches!(self.anno_stack.last(), Some(Anno::Backslash)) {
            // `Anno::Backslash` contains the source character `\`.
            // In `RenderAnnotated::push_annotation` below, we emit `\polBackslash` for this annotation.
            // This already renders the backslash character, so we do not render `s` here.
            Ok(())
        } else {
            self.upstream.write_all(s.as_bytes())
        }
    }

    fn fail_doc(&self) -> Self::Error {
        io::Error::other("Document failed to render")
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
            Anno::Reference { module_uri: _, name: _ } => "",
        };
        self.upstream.write_all(out.as_bytes())
    }

    fn pop_annotation(&mut self) -> Result<(), Self::Error> {
        let res = match self.anno_stack.last() {
            Some(Anno::BraceOpen)
            | Some(Anno::BraceClose)
            | Some(Anno::Reference { module_uri: _, name: _ }) => Ok(()),
            _ => self.upstream.write_all("}".as_bytes()),
        };
        self.anno_stack.pop();
        res
    }
}

#[cfg(test)]
mod tests {
    use pretty::{Render, RenderAnnotated};

    use super::*;

    #[test]
    fn backslash_is_rendered_to_proper_latex_command() {
        let mut output = Vec::new();
        let mut renderer = RenderLatex::new(&mut output);

        renderer.push_annotation(&Anno::Backslash).unwrap();
        renderer.write_str_all("\\").unwrap();
        renderer.pop_annotation().unwrap();

        assert_eq!(String::from_utf8(output).unwrap(), r"\polBackslash{}");
    }
}
