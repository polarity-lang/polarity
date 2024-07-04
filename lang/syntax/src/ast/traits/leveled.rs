use crate::ast::{Idx, Lvl, Var};

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
