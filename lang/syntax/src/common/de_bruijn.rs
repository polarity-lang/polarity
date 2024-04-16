use std::fmt;
use std::ops::{Bound, RangeBounds};
use std::rc::Rc;

/// Two-dimensional De-Bruijn index
///
/// The first component counts the number of binder lists in scope between the variable
/// and the binder list it originated from.
/// The second component counts the number of binders in that binder list between the end
/// of the binder list and the binder this variable originated from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Idx {
    pub fst: usize,
    pub snd: usize,
}

/// Two-dimensional De-Bruijn level
///
/// The first component counts the number of binder lists in scope between the root of the
/// term and the binder list it originated from.
/// The second component counts the number of binders in that binder list between the start
/// of the binder list and the binder this variable originated from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Lvl {
    pub fst: usize,
    pub snd: usize,
}

impl Lvl {
    pub fn here() -> Self {
        Self { fst: 0, snd: 0 }
    }
}

/// Either a De-Bruijn level or an index
///
/// Used to support lookup with both representations using the same interface
#[derive(Debug, Clone, Copy)]
pub enum Var {
    Lvl(Lvl),
    Idx(Idx),
}

impl From<Idx> for Var {
    fn from(idx: Idx) -> Self {
        Var::Idx(idx)
    }
}

impl From<Lvl> for Var {
    fn from(lvl: Lvl) -> Self {
        Var::Lvl(lvl)
    }
}

/// Convert an De-Bruijn index to a De-Bruijn level
///
/// To perform this conversion, it is sufficient to track
/// * the current number of De-Bruijn levels (maximum first component)
/// * the current number of binders per De-Bruijn level (maximum second component for each first component).
/// This is satisfied by the context type during typechecking.
pub trait Leveled {
    fn idx_to_lvl(&self, idx: Idx) -> Lvl;
    fn lvl_to_idx(&self, lvl: Lvl) -> Idx;
    fn var_to_lvl(&self, var: Var) -> Lvl {
        match var {
            Var::Lvl(lvl) => lvl,
            Var::Idx(idx) => self.idx_to_lvl(idx),
        }
    }
    fn var_to_idx(&self, var: Var) -> Idx {
        match var {
            Var::Lvl(lvl) => self.lvl_to_idx(lvl),
            Var::Idx(idx) => idx,
        }
    }
}

/// De-Bruijn shifting
///
/// When we manipulate terms using de Bruijn notation we often
/// have to change the de Bruijn indices of the variables inside
/// a term. This is what the "shift" and "shift_in_range" functions
/// from this trait are for.
///
/// Simplified Example: Consider the lambda calculus with de Bruijn
/// indices whose syntax is "e := n | λ_. e | e e". The shift_in_range
/// operation would be defined as follows:
/// - n.shift_in_range(range, by) = if (n ∈ range) then { n + by } else { n }
/// - (λ_. e).shift_in_range(range, by) = λ_.(e.shift_in_range(range.left += 1, by))
/// - (e1 e2).shift_in_range(range, by) = (e1.shift_in_range(range, by)) (e2.shift_in_range(range, by))
/// So whenever we traverse a binding occurrence we have to bump the left
/// side of the range by one.
///
/// Note: We use two-level de Bruijn indices. The cutoff-range only applies to
/// the first element of a two-level de Bruijn index.
///
/// Ref: https://www.cs.cornell.edu/courses/cs4110/2018fa/lectures/lecture15.pdf
pub trait Shift: Sized {
    /// Shift a term in the first component of the two-dimensional De-Bruijn index
    fn shift(&self, by: (isize, isize)) -> Self {
        self.shift_in_range(0.., by)
    }

    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self;
}

impl Shift for Idx {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        if range.contains(&self.fst) {
            Self {
                fst: (self.fst as isize + by.0) as usize,
                snd: (self.snd as isize + by.1) as usize,
            }
        } else {
            *self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shift_fst() {
        let result = Idx { fst: 0, snd: 0 }.shift((1, 0));
        assert_eq!(result, Idx { fst: 1, snd: 0 });
    }

    #[test]
    fn shift_snd() {
        let result = Idx { fst: 0, snd: 0 }.shift((0, 1));
        assert_eq!(result, Idx { fst: 0, snd: 1 });
    }

    #[test]
    fn shift_in_range_fst() {
        let result = Idx { fst: 0, snd: 0 }.shift_in_range(1.., (1, 0));
        assert_eq!(result, Idx { fst: 0, snd: 0 });
    }

    #[test]
    fn shift_in_range_snd() {
        let result = Idx { fst: 0, snd: 0 }.shift_in_range(1.., (0, 1));
        assert_eq!(result, Idx { fst: 0, snd: 0 });
    }
}

pub trait ShiftRange: RangeBounds<usize> + Clone {}

impl<T: RangeBounds<usize> + Clone> ShiftRange for T {}

impl Shift for () {
    fn shift_in_range<R: ShiftRange>(&self, _range: R, _by: (isize, isize)) -> Self {}
}

impl<T: Shift> Shift for Rc<T> {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        Rc::new((**self).shift_in_range(range, by))
    }
}

impl<T: Shift> Shift for Option<T> {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        self.as_ref().map(|inner| inner.shift_in_range(range, by))
    }
}

impl<T: Shift> Shift for Vec<T> {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        self.iter().map(|x| x.shift_in_range(range.clone(), by)).collect()
    }
}

pub trait ShiftRangeExt {
    type Target;

    fn shift(self, by: isize) -> Self::Target;
}

impl<R: ShiftRange> ShiftRangeExt for R {
    type Target = (Bound<usize>, Bound<usize>);

    fn shift(self, by: isize) -> Self::Target {
        fn shift_bound(bound: Bound<&usize>, by: isize) -> Bound<usize> {
            match bound {
                Bound::Included(x) => Bound::Included((*x as isize + by) as usize),
                Bound::Excluded(x) => Bound::Excluded((*x as isize + by) as usize),
                Bound::Unbounded => Bound::Unbounded,
            }
        }

        (shift_bound(self.start_bound(), by), shift_bound(self.end_bound(), by))
    }
}

impl fmt::Display for Idx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.fst, self.snd)
    }
}

impl fmt::Display for Lvl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.fst, self.snd)
    }
}

impl fmt::Display for Var {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Var::Lvl(lvl) => write!(f, "lvl:{lvl}"),
            Var::Idx(idx) => write!(f, "idx:{idx}"),
        }
    }
}
