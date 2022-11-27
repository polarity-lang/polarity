use crate::common::ShiftCutoff;

use super::def::*;

impl ShiftCutoff for Env {
    fn shift_cutoff(&self, cutoff: usize, by: (isize, isize)) -> Self {
        self.map(|val| val.shift_cutoff(cutoff, by))
    }
}
