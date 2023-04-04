use crate::common::*;

use super::def::*;

impl AlphaEq for Exp {
    fn alpha_eq(&self, other: &Self) -> bool {
        self == other
    }
}
