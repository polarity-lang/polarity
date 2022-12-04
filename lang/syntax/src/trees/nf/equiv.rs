use crate::common::*;

use super::def::*;

impl AlphaEq for Nf {
    fn alpha_eq(&self, other: &Self) -> bool {
        self == other
    }
}
