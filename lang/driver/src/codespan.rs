use miette_util::codespan::{
    ByteIndex, ColumnIndex, LineIndex, LineOffset, Location, RawIndex, Span,
};
use std::ffi::{OsStr, OsString};

/// An enum representing an error that happened while looking up a file or a piece of content in that file.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// A required file is not in the file database.
    FileMissing,
    /// The file is present, but does not contain the specified byte index.
    IndexTooLarge { given: usize, max: usize },
    /// The file is present, but does not contain the specified line index.
    LineTooLarge { given: usize, max: usize },
    /// The file is present and contains the specified line index, but the line does not contain the specified column index.
    ColumnTooLarge { given: usize, max: usize },
    /// The given index is contained in the file, but is not a boundary of a UTF-8 code point.
    InvalidCharBoundary { given: usize },
    /// There was a error while doing IO.
    Io(std::io::Error),
}

/// A file that is stored in the database.
#[derive(Debug, Clone)]
pub struct File<Source> {
    /// The name of the file.
    pub name: OsString,
    /// The source code of the file.
    pub source: Source,
    /// The starting byte indices in the source code.
    pub line_starts: Vec<ByteIndex>,
}

impl<Source> File<Source>
where
    Source: AsRef<str>,
{
    pub fn new(name: OsString, source: Source) -> Self {
        let line_starts = line_starts(source.as_ref()).map(|i| ByteIndex::from(i as u32)).collect();

        File { name, source, line_starts }
    }

    pub fn update(&mut self, source: Source) {
        let line_starts = line_starts(source.as_ref()).map(|i| ByteIndex::from(i as u32)).collect();
        self.source = source;
        self.line_starts = line_starts;
    }

    pub fn name(&self) -> &OsStr {
        &self.name
    }

    pub fn line_start(&self, line_index: LineIndex) -> Result<ByteIndex, Error> {
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

    pub fn last_line_index(&self) -> LineIndex {
        LineIndex::from(self.line_starts.len() as RawIndex)
    }

    pub fn line_span(&self, line_index: LineIndex) -> Result<Span, Error> {
        let line_start = self.line_start(line_index)?;
        let next_line_start = self.line_start(line_index + LineOffset::from(1))?;

        Ok(Span::new(line_start, next_line_start))
    }

    pub fn line_index(&self, byte_index: ByteIndex) -> LineIndex {
        match self.line_starts.binary_search(&byte_index) {
            // Found the start of a line
            Ok(line) => LineIndex::from(line as u32),
            Err(next_line) => LineIndex::from(next_line as u32 - 1),
        }
    }

    pub fn location(&self, byte_index: ByteIndex) -> Result<Location, Error> {
        let line_index = self.line_index(byte_index);
        let line_start_index = self.line_start(line_index).map_err(|_| Error::IndexTooLarge {
            given: byte_index.to_usize(),
            max: self.source().as_ref().len() - 1,
        })?;
        let line_src = self
            .source
            .as_ref()
            .get(line_start_index.to_usize()..byte_index.to_usize())
            .ok_or_else(|| {
                let given = byte_index.to_usize();
                let max = self.source().as_ref().len() - 1;
                if given > max {
                    Error::IndexTooLarge { given, max }
                } else {
                    Error::InvalidCharBoundary { given }
                }
            })?;

        Ok(Location {
            line: line_index,
            column: ColumnIndex::from(line_src.chars().count() as u32),
        })
    }

    pub fn source(&self) -> &Source {
        &self.source
    }

    pub fn source_span(&self) -> Span {
        Span::from_string(self.source.as_ref())
    }

    pub fn source_slice(&self, span: Span) -> Result<&str, Error> {
        let start = span.start().to_usize();
        let end = span.end().to_usize();

        self.source.as_ref().get(start..end).ok_or_else(|| {
            let max = self.source().as_ref().len() - 1;
            Error::IndexTooLarge { given: if start > max { start } else { end }, max }
        })
    }
}

// NOTE: this is copied from `codespan_reporting::files::line_starts` and should be kept in sync.
fn line_starts(source: &str) -> impl '_ + Iterator<Item = usize> {
    std::iter::once(0).chain(source.match_indices('\n').map(|(i, _)| i + 1))
}
