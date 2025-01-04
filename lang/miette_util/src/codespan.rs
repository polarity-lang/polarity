use std::fmt;
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

/// A 1-indexed line number. Useful for pretty printing source locations.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LineNumber(RawIndex);

impl fmt::Display for LineNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

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

impl fmt::Display for ColumnNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// A zero-indexed column offset into a source file
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ColumnIndex(pub RawIndex);

/// A column offset in a source file
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ColumnOffset(pub RawOffset);

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
