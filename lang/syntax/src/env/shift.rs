use crate::common::*;

use super::def::*;

impl ShiftInRange for Env {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        self.map(|val| val.shift_in_range(range.clone(), by))
    }
}
