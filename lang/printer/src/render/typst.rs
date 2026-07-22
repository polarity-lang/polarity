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

impl<W> RenderTypst<W>
where
    W: io::Write,
{
    /// All Polarity code is wrapped in Typst string literals.
    /// We thereby escape special characters that may otherwise be misinterpreted by Typst.
    fn write_text(&mut self, text: &str) -> io::Result<()> {
        let mut rest = text;

        while let Some((line, after_line)) = rest.split_once('\n') {
            self.write_line(line)?;
            self.upstream.write_all(b"#linebreak()\n")?;
            rest = after_line;
        }

        self.write_line(rest)
    }

    fn write_line(&mut self, line: &str) -> io::Result<()> {
        if line.is_empty() {
            return Ok(());
        }

        self.upstream.write_all(b"#\"")?;
        for character in line.chars() {
            match character {
                '\\' => self.upstream.write_all(b"\\\\")?,
                '"' => self.upstream.write_all(b"\\\"")?,
                character => {
                    let mut encoded = [0; 4];
                    self.upstream.write_all(character.encode_utf8(&mut encoded).as_bytes())?;
                }
            }
        }
        self.upstream.write_all(b"\"")
    }
}

impl<W> pretty::Render for RenderTypst<W>
where
    W: io::Write,
{
    type Error = io::Error;

    fn write_str(&mut self, s: &str) -> io::Result<usize> {
        self.write_str_all(s).map(|()| s.len())
    }

    fn write_str_all(&mut self, s: &str) -> io::Result<()> {
        self.write_text(s)
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
        let out = match anno {
            Anno::Keyword => "#text(fill: blue)[",
            Anno::Ctor => "#text(fill: red)[",
            Anno::Dtor => "#text(fill: green)[",
            Anno::Type => "#text(fill: olive)[",
            Anno::Comment => "#text(fill: maroon)[",
            Anno::Error => "#text(fill: red)[",
            Anno::Backslash | Anno::BraceOpen | Anno::BraceClose | Anno::Reference { .. } => "",
        };
        self.anno_stack.push(anno.clone());
        self.upstream.write_all(out.as_bytes())
    }

    fn pop_annotation(&mut self) -> Result<(), Self::Error> {
        let out = match self.anno_stack.pop() {
            Some(
                Anno::Keyword | Anno::Ctor | Anno::Dtor | Anno::Type | Anno::Comment | Anno::Error,
            ) => "]",
            Some(Anno::Backslash | Anno::BraceOpen | Anno::BraceClose | Anno::Reference { .. })
            | None => "",
        };
        self.upstream.write_all(out.as_bytes())
    }
}
