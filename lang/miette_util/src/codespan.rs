//! The code in this module was originally part of the codespan library https://github.com/brendanzab/codespan.
//!
//! The contributors @Johann150, @Marwes, @brendanzab, @jyn514 and @etaoins have graciously
//! agreed to additionally license their contributions under both the Apache-2.0 and the MIT license to us:
//! - https://github.com/polarity-lang/polarity/pull/425
use std::ops::{Add, Sub};

/// A byte position in a source file.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ByteIndex(pub u32);

impl ByteIndex {
    /// Convert the position into a `usize`, for use in array indexing
    pub const fn to_usize(self) -> usize {
        self.0 as usize
    }
}

impl Sub for ByteIndex {
    type Output = ByteOffset;

    #[inline]
    fn sub(self, rhs: ByteIndex) -> ByteOffset {
        ByteOffset(self.0 as i64 - rhs.0 as i64)
    }
}

impl Add<ByteOffset> for ByteIndex {
    type Output = ByteIndex;

    #[inline]
    fn add(self, rhs: ByteOffset) -> ByteIndex {
        ByteIndex((self.0 as i64 + rhs.0) as u32)
    }
}

/// A byte offset in a source file
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ByteOffset(pub i64);

impl ByteOffset {
    /// Convert the offset into a `usize`, for use in array indexing
    pub const fn to_usize(self) -> usize {
        self.0 as usize
    }
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Span {
    pub start: ByteIndex,
    pub end: ByteIndex,
}

impl Span {
    /// Gives an empty span at the start of a source.
    pub const fn initial() -> Span {
        Span { start: ByteIndex(0), end: ByteIndex(0) }
    }

    /// Measure the span of a string.
    pub fn from_string(s: &str) -> Span {
        Span { start: ByteIndex(0), end: ByteIndex(s.len() as u32) }
    }
}

/// A 1-indexed line number. Useful for pretty printing source locations.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LineNumber(u32);

/// A zero-indexed line offset into a source file
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LineIndex(pub u32);

/// A line offset in a source file
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LineOffset(pub i64);

impl LineIndex {
    /// The 1-indexed line number. Useful for pretty printing source locations.
    pub const fn number(self) -> LineNumber {
        LineNumber(self.0 + 1)
    }

    /// Convert the index into a `usize`, for use in array indexing
    pub const fn to_usize(self) -> usize {
        self.0 as usize
    }
}

impl Add<LineOffset> for LineIndex {
    type Output = LineIndex;

    #[inline]
    fn add(self, rhs: LineOffset) -> LineIndex {
        LineIndex((self.0 as i64 + rhs.0) as u32)
    }
}

/// A 1-indexed column number. Useful for pretty printing source locations.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ColumnNumber(u32);

/// A zero-indexed column offset into a source file
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ColumnIndex(pub u32);

/// A column offset in a source file
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ColumnOffset(pub i64);

impl ColumnIndex {
    /// Convert the index into a `usize`, for use in array indexing
    pub const fn to_usize(self) -> usize {
        self.0 as usize
    }
}
