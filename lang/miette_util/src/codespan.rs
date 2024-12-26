use std::fmt;
use std::ops::Range;
use std::ops::{Add, AddAssign, Neg, Sub, SubAssign};

/// The raw, untyped offset.
pub type RawOffset = i64;

/// The raw, untyped index. We use a 32-bit integer here for space efficiency,
/// assuming we won't be working with sources larger than 4GB.
pub type RawIndex = u32;

/// A byte position in a source file.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ByteIndex(pub RawIndex);

impl ByteIndex {
    /// Convert the position into a `usize`, for use in array indexing
    pub const fn to_usize(self) -> usize {
        self.0 as usize
    }
}

#[allow(clippy::derivable_impls)]
impl Default for ByteIndex {
    fn default() -> ByteIndex {
        ByteIndex(0)
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

/// A byte offset in a source file
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ByteOffset(pub RawOffset);

impl ByteOffset {
    /// Create a byte offset from a UTF8-encoded character
    ///
    /// ```rust
    /// use miette_util::codespan::ByteOffset;
    ///
    /// assert_eq!(ByteOffset::from_char_len('A').to_usize(), 1);
    /// assert_eq!(ByteOffset::from_char_len('ß').to_usize(), 2);
    /// assert_eq!(ByteOffset::from_char_len('ℝ').to_usize(), 3);
    /// assert_eq!(ByteOffset::from_char_len('💣').to_usize(), 4);
    /// ```
    pub fn from_char_len(ch: char) -> ByteOffset {
        ByteOffset(ch.len_utf8() as RawOffset)
    }

    /// Create a byte offset from a UTF- encoded string
    ///
    /// ```rust
    /// use miette_util::codespan::ByteOffset;
    ///
    /// assert_eq!(ByteOffset::from_str_len("A").to_usize(), 1);
    /// assert_eq!(ByteOffset::from_str_len("ß").to_usize(), 2);
    /// assert_eq!(ByteOffset::from_str_len("ℝ").to_usize(), 3);
    /// assert_eq!(ByteOffset::from_str_len("💣").to_usize(), 4);
    /// ```
    pub fn from_str_len(value: &str) -> ByteOffset {
        ByteOffset(value.len() as RawOffset)
    }

    /// Convert the offset into a `usize`, for use in array indexing
    pub const fn to_usize(self) -> usize {
        self.0 as usize
    }
}

impl Default for ByteOffset {
    #[inline]
    fn default() -> ByteOffset {
        ByteOffset(0)
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
    pub fn new(start: impl Into<ByteIndex>, end: impl Into<ByteIndex>) -> Span {
        let start = start.into();
        let end = end.into();

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
    /// assert_eq!(span, Span::new(0, 5));
    /// ```
    pub fn from_string(s: &str) -> Span {
        Span::new(0, s.len() as u32)
    }

    /// Combine two spans by taking the start of the earlier span
    /// and the end of the later span.
    ///
    /// Note: this will work even if the two spans are disjoint.
    /// If this doesn't make sense in your application, you should handle it yourself.
    /// In that case, you can use `Span::disjoint` as a convenience function.
    ///
    /// ```rust
    /// use miette_util::codespan::Span;
    ///
    /// let span1 = Span::new(0, 4);
    /// let span2 = Span::new(10, 16);
    ///
    /// assert_eq!(Span::merge(span1, span2), Span::new(0, 16));
    /// ```
    pub fn merge(self, other: Span) -> Span {
        use std::cmp::{max, min};

        let start = min(self.start, other.start);
        let end = max(self.end, other.end);
        Span::new(start, end)
    }

    /// A helper function to tell whether two spans do not overlap.
    ///
    /// ```
    /// use miette_util::codespan::Span;
    /// let span1 = Span::new(0, 4);
    /// let span2 = Span::new(10, 16);
    /// assert!(span1.disjoint(span2));
    /// ```
    pub fn disjoint(self, other: Span) -> bool {
        let (first, last) = if self.end < other.end { (self, other) } else { (other, self) };
        first.end <= last.start
    }

    /// Get the starting byte index.
    ///
    /// ```rust
    /// use miette_util::codespan::{ByteIndex, Span};
    ///
    /// let span = Span::new(0, 4);
    ///
    /// assert_eq!(span.start(), ByteIndex::from(0));
    /// ```
    pub fn start(self) -> ByteIndex {
        self.start
    }

    /// Get the ending byte index.
    ///
    /// ```rust
    /// use miette_util::codespan::{ByteIndex, Span};
    ///
    /// let span = Span::new(0, 4);
    ///
    /// assert_eq!(span.end(), ByteIndex::from(4));
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

impl<I> From<Range<I>> for Span
where
    I: Into<ByteIndex>,
{
    fn from(range: Range<I>) -> Span {
        Span::new(range.start, range.end)
    }
}

impl From<Span> for Range<usize> {
    fn from(span: Span) -> Range<usize> {
        span.start.into()..span.end.into()
    }
}

impl From<Span> for Range<RawIndex> {
    fn from(span: Span) -> Range<RawIndex> {
        span.start.0..span.end.0
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_merge() {
        use super::Span;

        // overlap
        let a = Span::from(1..5);
        let b = Span::from(3..10);
        assert_eq!(a.merge(b), Span::from(1..10));
        assert_eq!(b.merge(a), Span::from(1..10));

        // subset
        let two_four = (2..4).into();
        assert_eq!(a.merge(two_four), (1..5).into());
        assert_eq!(two_four.merge(a), (1..5).into());

        // disjoint
        let ten_twenty = (10..20).into();
        assert_eq!(a.merge(ten_twenty), (1..20).into());
        assert_eq!(ten_twenty.merge(a), (1..20).into());

        // identity
        assert_eq!(a.merge(a), a);
    }

    #[test]
    fn test_disjoint() {
        use super::Span;

        // overlap
        let a = Span::from(1..5);
        let b = Span::from(3..10);
        assert!(!a.disjoint(b));
        assert!(!b.disjoint(a));

        // subset
        let two_four = (2..4).into();
        assert!(!a.disjoint(two_four));
        assert!(!two_four.disjoint(a));

        // disjoint
        let ten_twenty = (10..20).into();
        assert!(a.disjoint(ten_twenty));
        assert!(ten_twenty.disjoint(a));

        // identity
        assert!(!a.disjoint(a));

        // off by one (upper bound)
        let c = Span::from(5..10);
        assert!(a.disjoint(c));
        assert!(c.disjoint(a));
        // off by one (lower bound)
        let d = Span::from(0..1);
        assert!(a.disjoint(d));
        assert!(d.disjoint(a));
    }
}

/// A 1-indexed line number. Useful for pretty printing source locations.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LineNumber(RawIndex);

/// A zero-indexed line offset into a source file
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LineIndex(pub RawIndex);

/// A line offset in a source file
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LineOffset(pub RawOffset);

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

#[allow(clippy::derivable_impls)]
impl Default for LineIndex {
    fn default() -> LineIndex {
        LineIndex(0)
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
pub struct ColumnNumber(RawIndex);

/// A zero-indexed column offset into a source file
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ColumnIndex(pub RawIndex);

/// A column offset in a source file
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ColumnOffset(pub RawOffset);

impl ColumnIndex {
    /// The 1-indexed column number. Useful for pretty printing source locations.
    ///
    /// ```rust
    /// use miette_util::codespan::{ColumnIndex, ColumnNumber};
    ///
    /// assert_eq!(format!("{}", ColumnIndex(0).number()), "1");
    /// assert_eq!(format!("{}", ColumnIndex(3).number()), "4");
    /// ```
    pub const fn number(self) -> ColumnNumber {
        ColumnNumber(self.0 + 1)
    }

    /// Convert the index into a `usize`, for use in array indexing
    pub const fn to_usize(self) -> usize {
        self.0 as usize
    }
}

#[allow(clippy::derivable_impls)]
impl Default for ColumnIndex {
    fn default() -> ColumnIndex {
        ColumnIndex(0)
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
/// A location in a source file.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Location {
    /// The line index in the source file.
    pub line: LineIndex,
    /// The column index in the source file.
    pub column: ColumnIndex,
}

impl Location {
    /// Construct a new location from a line index and a column index.
    pub fn new(line: impl Into<LineIndex>, column: impl Into<ColumnIndex>) -> Location {
        Location { line: line.into(), column: column.into() }
    }
}

/// A relative offset between two indices
///
/// These can be thought of as 1-dimensional vectors
pub trait Offset: Copy + Ord
where
    Self: Neg<Output = Self>,
    Self: Add<Self, Output = Self>,
    Self: AddAssign<Self>,
    Self: Sub<Self, Output = Self>,
    Self: SubAssign<Self>,
{
    const ZERO: Self;
}

/// Index types
///
/// These can be thought of as 1-dimensional points
pub trait Index: Copy + Ord
where
    Self: Add<<Self as Index>::Offset, Output = Self>,
    Self: AddAssign<<Self as Index>::Offset>,
    Self: Sub<<Self as Index>::Offset, Output = Self>,
    Self: SubAssign<<Self as Index>::Offset>,
    Self: Sub<Self, Output = <Self as Index>::Offset>,
{
    type Offset: Offset;
}

macro_rules! impl_index {
    ($Index:ident, $Offset:ident) => {
        impl From<RawOffset> for $Offset {
            #[inline]
            fn from(i: RawOffset) -> Self {
                $Offset(i)
            }
        }

        impl From<RawIndex> for $Index {
            #[inline]
            fn from(i: RawIndex) -> Self {
                $Index(i)
            }
        }

        impl From<$Index> for RawIndex {
            #[inline]
            fn from(index: $Index) -> RawIndex {
                index.0
            }
        }

        impl From<$Offset> for RawOffset {
            #[inline]
            fn from(offset: $Offset) -> RawOffset {
                offset.0
            }
        }

        impl From<$Index> for usize {
            #[inline]
            fn from(index: $Index) -> usize {
                index.0 as usize
            }
        }

        impl From<$Offset> for usize {
            #[inline]
            fn from(offset: $Offset) -> usize {
                offset.0 as usize
            }
        }

        impl Offset for $Offset {
            const ZERO: $Offset = $Offset(0);
        }

        impl Index for $Index {
            type Offset = $Offset;
        }

        impl Add<$Offset> for $Index {
            type Output = $Index;

            #[inline]
            fn add(self, rhs: $Offset) -> $Index {
                $Index((self.0 as RawOffset + rhs.0) as RawIndex)
            }
        }

        impl AddAssign<$Offset> for $Index {
            #[inline]
            fn add_assign(&mut self, rhs: $Offset) {
                *self = *self + rhs;
            }
        }

        impl Neg for $Offset {
            type Output = $Offset;

            #[inline]
            fn neg(self) -> $Offset {
                $Offset(-self.0)
            }
        }

        impl Add<$Offset> for $Offset {
            type Output = $Offset;

            #[inline]
            fn add(self, rhs: $Offset) -> $Offset {
                $Offset(self.0 + rhs.0)
            }
        }

        impl AddAssign<$Offset> for $Offset {
            #[inline]
            fn add_assign(&mut self, rhs: $Offset) {
                self.0 += rhs.0;
            }
        }

        impl Sub<$Offset> for $Offset {
            type Output = $Offset;

            #[inline]
            fn sub(self, rhs: $Offset) -> $Offset {
                $Offset(self.0 - rhs.0)
            }
        }

        impl SubAssign<$Offset> for $Offset {
            #[inline]
            fn sub_assign(&mut self, rhs: $Offset) {
                self.0 -= rhs.0;
            }
        }

        impl Sub for $Index {
            type Output = $Offset;

            #[inline]
            fn sub(self, rhs: $Index) -> $Offset {
                $Offset(self.0 as RawOffset - rhs.0 as RawOffset)
            }
        }

        impl Sub<$Offset> for $Index {
            type Output = $Index;

            #[inline]
            fn sub(self, rhs: $Offset) -> $Index {
                $Index((self.0 as RawOffset - rhs.0 as RawOffset) as u32)
            }
        }

        impl SubAssign<$Offset> for $Index {
            #[inline]
            fn sub_assign(&mut self, rhs: $Offset) {
                self.0 = (self.0 as RawOffset - rhs.0) as RawIndex;
            }
        }
    };
}

impl_index!(ByteIndex, ByteOffset);
impl_index!(LineIndex, LineOffset);
impl_index!(ColumnIndex, ColumnOffset);
