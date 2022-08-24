use std::ops;

#[cfg(feature = "use-serde")]
use serde_derive::{Deserialize, Serialize};

/// Two-level De-Bruijn index
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
#[cfg_attr(feature = "use-serde", derive(Serialize, Deserialize))]
pub struct Idx {
    pub lvl: usize,
    pub idx: usize,
}

impl Idx {
    pub fn here() -> Self {
        Self { lvl: 0, idx: 0 }
    }

    pub fn map_lvl<F: FnOnce(usize) -> usize>(self, f: F) -> Self {
        Self { lvl: f(self.lvl), idx: self.idx }
    }

    pub fn map_idx<F: FnOnce(usize) -> usize>(self, f: F) -> Self {
        Self { lvl: self.lvl, idx: f(self.idx) }
    }
}

impl ops::Sub for Idx {
    type Output = Idx;

    fn sub(self, rhs: Self) -> Self::Output {
        Self { lvl: self.lvl - rhs.lvl, idx: self.idx - rhs.idx }
    }
}
