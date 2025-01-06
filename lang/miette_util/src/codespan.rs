use std::fmt;
use std::ops::{Add, Sub};

/// A byte position in a source file.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ByteIndex(pub u32);

impl ByteIndex {
    /// Convert the position into a `usize`, for use in array indexing
    pub const fn to_usize(self) -> usize {
        self.0 as usize
    }
}

impl fmt::Debug for ByteIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ByteIndex(")?;
        self.0.fmt(f)?;
        write!(f, ")")
    }
}

impl fmt::Display for ByteIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
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
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ByteOffset(pub i64);

impl ByteOffset {
    /// Convert the offset into a `usize`, for use in array indexing
    pub const fn to_usize(self) -> usize {
        self.0 as usize
    }
}

impl fmt::Debug for ByteOffset {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ByteOffset(")?;
        self.0.fmt(f)?;
        write!(f, ")")
    }
}

impl fmt::Display for ByteOffset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Span {
    start: ByteIndex,
    end: ByteIndex,
}

impl Span {
    /// Create a new span from a starting and ending span.
    pub fn new(start: ByteIndex, end: ByteIndex) -> Span {
        assert!(end >= start);

        Span { start, end }
    }

    /// Gives an empty span at the start of a source.
    pub const fn initial() -> Span {
        Span { start: ByteIndex(0), end: ByteIndex(0) }
    }

    /// Measure the span of a string.
    ///
    /// ```rust
    /// use miette_util::codespan::{ByteIndex, Span};
    ///
    /// let span = Span::from_string("hello");
    ///
    /// assert_eq!(span, Span::new(ByteIndex(0), ByteIndex(5)));
    /// ```
    pub fn from_string(s: &str) -> Span {
        Span::new(ByteIndex(0), ByteIndex(s.len() as u32))
    }

    /// Get the starting byte index.
    ///
    /// ```rust
    /// use miette_util::codespan::{ByteIndex, Span};
    ///
    /// let span = Span::new(ByteIndex(0), ByteIndex(4));
    ///
    /// assert_eq!(span.start(), ByteIndex(0));
    /// ```
    pub fn start(self) -> ByteIndex {
        self.start
    }

    /// Get the ending byte index.
    ///
    /// ```rust
    /// use miette_util::codespan::{ByteIndex, Span};
    ///
    /// let span = Span::new(ByteIndex(0), ByteIndex(4));
    ///
    /// assert_eq!(span.end(), ByteIndex(4));
    /// ```
    pub fn end(self) -> ByteIndex {
        self.end
    }
}

impl Default for Span {
    fn default() -> Span {
        Span::initial()
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{start}, {end})", start = self.start(), end = self.end(),)
    }
}

/// A 1-indexed line number. Useful for pretty printing source locations.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LineNumber(u32);

impl fmt::Display for LineNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// A zero-indexed line offset into a source file
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LineIndex(pub u32);

/// A line offset in a source file
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LineOffset(pub i64);

impl LineIndex {
    /// The 1-indexed line number. Useful for pretty printing source locations.
    ///
    /// ```rust
    /// use miette_util::codespan::{LineIndex, LineNumber};
    ///
    /// assert_eq!(format!("{}", LineIndex(0).number()), "1");
    /// assert_eq!(format!("{}", LineIndex(3).number()), "4");
    /// ```
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

impl fmt::Debug for LineIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "LineIndex(")?;
        self.0.fmt(f)?;
        write!(f, ")")
    }
}

impl fmt::Display for LineIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// A 1-indexed column number. Useful for pretty printing source locations.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ColumnNumber(u32);

impl fmt::Display for ColumnNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// A zero-indexed column offset into a source file
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
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

impl fmt::Debug for ColumnIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ColumnIndex(")?;
        self.0.fmt(f)?;
        write!(f, ")")
    }
}

impl fmt::Display for ColumnIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
