use std::ops::{Bound, RangeBounds};
use std::rc::Rc;

use crate::ast::*;

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
///
/// So whenever we traverse a binding occurrence we have to bump the left
/// side of the range by one.
///
/// Note: We use two-level de Bruijn indices. The cutoff-range only applies to
/// the first element of a two-level de Bruijn index.
///
/// Ref: <https://www.cs.cornell.edu/courses/cs4110/2018fa/lectures/lecture15.pdf>
pub trait Shift: Sized {
    /// Shift all open variables in `self` by the the value indicated with the
    /// `by` argument.
    fn shift(&self, by: (isize, isize)) -> Self {
        self.shift_in_range(0.., by)
    }

    /// Shift every de Bruijn index contained in `self` by the value indicated
    /// with the `by` argument. De Bruijn indices whose first component does not
    /// lie within the indicated `range` are not affected by the shift.
    ///
    /// In order to implement `shift_in_range` correctly you have to increase the
    /// left endpoint of `range` by 1 whenever you go recursively under a binder.
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
