use std::io;

use crate::types::*;

pub struct RenderTypst<W> {
    anno_stack: Vec<Anno>,
    upstream: W,
}

impl<W> RenderTypst<W> {
    pub fn new(upstream: W) -> RenderTypst<W> {
        RenderTypst { anno_stack: Vec::new(), upstream }
    }
}

/// - Replace `\n` line endings by explicit `#linebreak()` functions.
/// - Replace whitespace by `#h(0.1cm)`
fn make_layout_explicit(s: &str) -> String {
    s.to_string().replace('\n', "#linebreak()\n").replace(' ', "#h(0.1cm)").replace('_', "\\_")
}

impl<W> pretty::Render for RenderTypst<W>
where
    W: io::Write,
{
    type Error = io::Error;

    fn write_str(&mut self, s: &str) -> io::Result<usize> {
        let s_replaced = make_layout_explicit(s);
        self.upstream.write(s_replaced.as_bytes())
    }

    fn write_str_all(&mut self, s: &str) -> io::Result<()> {
        let s_replaced = make_layout_explicit(s);
        self.upstream.write_all(s_replaced.as_bytes())
    }

    fn fail_doc(&self) -> Self::Error {
        io::Error::other("Document failed to render")
    }
}

impl<W> pretty::RenderAnnotated<'_, Anno> for RenderTypst<W>
where
    W: io::Write,
{
    fn push_annotation(&mut self, anno: &Anno) -> Result<(), Self::Error> {
        self.anno_stack.push(anno.clone());
        let out = match anno {
            Anno::Keyword => r"#text(blue)[",
            Anno::Ctor => r"#text(red)[",
            Anno::Dtor => r"#text(green)[",
            Anno::Type => r"#text(olive)[",
            Anno::Comment => r"#text(maroon)[",
            _ => "",
        };
        self.upstream.write_all(out.as_bytes())
    }

    fn pop_annotation(&mut self) -> Result<(), Self::Error> {
        let res = match self.anno_stack.last() {
            Some(Anno::Keyword) | Some(Anno::Ctor) | Some(Anno::Dtor) | Some(Anno::Type)
            | Some(Anno::Comment) => self.upstream.write_all("] ".as_bytes()),
            _ => Ok(()),
        };
        self.anno_stack.pop();
        res
    }
}
