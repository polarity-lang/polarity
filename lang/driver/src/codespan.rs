//! The code in this module was originally part of the codespan library <https://github.com/brendanzab/codespan>.
//!
//! The codespan contributors @Johann150, @Marwes, @brendanzab, @jyn514 and @etaoins have graciously
//! agreed to additionally license their contributions under both the Apache-2.0 and the MIT license to us:
//! - <https://github.com/polarity-lang/polarity/pull/425>
use lsp_types::Position;
use miette_util::codespan::{ByteIndex, LineIndex, LineOffset, Span};

use crate::DriverError;

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
        let line_starts: Vec<ByteIndex> = std::iter::once(0)
            .chain(source.match_indices('\n').map(|(i, _)| i + 1))
            .map(|i| ByteIndex(i as u32))
            .collect();

        File { source, line_starts }
    }

    fn line_start(&self, line_index: LineIndex) -> Result<ByteIndex, DriverError> {
        use std::cmp::Ordering;

        match line_index.cmp(&self.last_line_index()) {
            Ordering::Less => Ok(self.line_starts[line_index.to_usize()]),
            Ordering::Equal => Ok(Span::from_string(self.source.as_ref()).end),
            Ordering::Greater => Err(DriverError::LineTooLarge {
                given: line_index.to_usize(),
                max: self.last_line_index().to_usize(),
            }),
        }
    }

    fn last_line_index(&self) -> LineIndex {
        LineIndex(self.line_starts.len() as u32)
    }

    pub fn line_span(&self, line_index: LineIndex) -> Result<Span, DriverError> {
        let line_start = self.line_start(line_index)?;
        let next_line_start = self.line_start(line_index + LineOffset(1))?;

        Ok(Span { start: line_start, end: next_line_start })
    }

    fn line_index(&self, byte_index: ByteIndex) -> LineIndex {
        match self.line_starts.binary_search(&byte_index) {
            // Found the start of a line
            Ok(line) => LineIndex(line as u32),
            Err(next_line) => LineIndex(next_line as u32 - 1),
        }
    }

    pub fn location(&self, byte_index: ByteIndex) -> Result<Position, DriverError> {
        let line_index = self.line_index(byte_index);
        let line_start_index = self.line_start(line_index).map_err(|_| {
            DriverError::IndexTooLarge { given: byte_index.to_usize(), max: self.source.len() - 1 }
        })?;
        let line_src = self
            .source
            .get(line_start_index.to_usize()..byte_index.to_usize())
            .ok_or_else(|| {
                let given = byte_index.to_usize();
                let max = self.source.len() - 1;
                if given > max {
                    DriverError::IndexTooLarge { given, max }
                } else {
                    DriverError::InvalidCharBoundary { given }
                }
            })?;

        Ok(Position { line: line_index.0, character: line_src.chars().count() as u32 })
    }
}
