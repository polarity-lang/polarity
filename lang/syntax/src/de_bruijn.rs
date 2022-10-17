use std::fmt;
use std::rc::Rc;

#[cfg(feature = "use-serde")]
use serde_derive::{Deserialize, Serialize};

use super::equiv::*;

/// Two-dimensional De-Bruijn index
///
/// The first component counts the number of binder lists in scope between the variable
/// and the binder list it originated from.
/// The second component counts the number of binders in that binder list between the end
/// of the binder list and the binder this variable originated from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "use-serde", derive(Serialize, Deserialize))]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Lvl {
    pub fst: usize,
    pub snd: usize,
}

impl Lvl {
    pub fn here() -> Self {
        Self { fst: 0, snd: 0 }
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
}

/// De-Bruijn shifting
pub trait Shift {
    /// Shift a term in the first component of the two-dimensional De-Bruijn index
    fn shift(&self, by: (isize, isize)) -> Self;
}

pub trait ShiftCutoff {
    fn shift_cutoff(&self, cutoff: usize, by: (isize, isize)) -> Self;
}

impl ShiftCutoff for Idx {
    fn shift_cutoff(&self, cutoff: usize, by: (isize, isize)) -> Self {
        if self.fst >= cutoff {
            Self {
                fst: (self.fst as isize + by.0) as usize,
                snd: (self.snd as isize + by.1) as usize,
            }
        } else {
            *self
        }
    }
}

impl<T: ShiftCutoff> ShiftCutoff for Rc<T> {
    fn shift_cutoff(&self, cutoff: usize, by: (isize, isize)) -> Self {
        Rc::new((**self).shift_cutoff(cutoff, by))
    }
}

impl<T: ShiftCutoff> ShiftCutoff for Vec<T> {
    fn shift_cutoff(&self, cutoff: usize, by: (isize, isize)) -> Self {
        self.iter().map(|x| x.shift_cutoff(cutoff, by)).collect()
    }
}

impl<T: ShiftCutoff> Shift for T {
    fn shift(&self, by: (isize, isize)) -> Self {
        self.shift_cutoff(0, by)
    }
}

impl AlphaEq for Idx {
    fn alpha_eq(&self, other: &Self) -> bool {
        self.fst == other.fst && self.snd == other.snd
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
