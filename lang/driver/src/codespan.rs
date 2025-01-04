use miette_util::codespan::{
    ByteIndex, ColumnIndex, LineIndex, LineOffset, Location, RawIndex, Span,
};

/// An enum representing an error that happened while looking up a file or a piece of content in that file.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// The file is present, but does not contain the specified byte index.
    IndexTooLarge { given: usize, max: usize },
    /// The file is present, but does not contain the specified line index.
    LineTooLarge { given: usize, max: usize },
    /// The given index is contained in the file, but is not a boundary of a UTF-8 code point.
    InvalidCharBoundary { given: usize },
}

/// A file that is stored in the database.
#[derive(Debug, Clone)]
pub struct File {
    /// The source code of the file.
    pub source: String,
    /// The starting byte indices in the source code.
    pub line_starts: Vec<ByteIndex>,
}

impl File {
    pub fn new(source: String) -> Self {
        let line_starts = line_starts(source.as_ref()).map(|i| ByteIndex(i as u32)).collect();

        File { source, line_starts }
    }

    fn line_start(&self, line_index: LineIndex) -> Result<ByteIndex, Error> {
        use std::cmp::Ordering;

        match line_index.cmp(&self.last_line_index()) {
            Ordering::Less => Ok(self.line_starts[line_index.to_usize()]),
            Ordering::Equal => Ok(self.source_span().end()),
            Ordering::Greater => Err(Error::LineTooLarge {
                given: line_index.to_usize(),
                max: self.last_line_index().to_usize(),
            }),
        }
    }

    fn last_line_index(&self) -> LineIndex {
        LineIndex(self.line_starts.len() as RawIndex)
    }

    pub fn line_span(&self, line_index: LineIndex) -> Result<Span, Error> {
        let line_start = self.line_start(line_index)?;
        let next_line_start = self.line_start(line_index + LineOffset(1))?;

        Ok(Span::new(line_start, next_line_start))
    }

    fn line_index(&self, byte_index: ByteIndex) -> LineIndex {
        match self.line_starts.binary_search(&byte_index) {
            // Found the start of a line
            Ok(line) => LineIndex(line as u32),
            Err(next_line) => LineIndex(next_line as u32 - 1),
        }
    }

    pub fn location(&self, byte_index: ByteIndex) -> Result<Location, Error> {
        let line_index = self.line_index(byte_index);
        let line_start_index = self.line_start(line_index).map_err(|_| Error::IndexTooLarge {
            given: byte_index.to_usize(),
            max: self.source().len() - 1,
        })?;
        let line_src = self
            .source
            .get(line_start_index.to_usize()..byte_index.to_usize())
            .ok_or_else(|| {
                let given = byte_index.to_usize();
                let max = self.source().len() - 1;
                if given > max {
                    Error::IndexTooLarge { given, max }
                } else {
                    Error::InvalidCharBoundary { given }
                }
            })?;

        Ok(Location { line: line_index, column: ColumnIndex(line_src.chars().count() as u32) })
    }

    pub fn source(&self) -> &String {
        &self.source
    }

    fn source_span(&self) -> Span {
        Span::from_string(self.source.as_ref())
    }
}

// NOTE: this is copied from `codespan_reporting::files::line_starts` and should be kept in sync.
fn line_starts(source: &str) -> impl '_ + Iterator<Item = usize> {
    std::iter::once(0).chain(source.match_indices('\n').map(|(i, _)| i + 1))
}
