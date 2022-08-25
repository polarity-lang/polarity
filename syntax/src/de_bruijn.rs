#[cfg(feature = "use-serde")]
use serde_derive::{Deserialize, Serialize};

/// Two-dimensional De-Bruijn index
///
/// The first component counts the number of binder lists in scope between the variable
/// and the binder list it originated from.
/// The second component counts the number of binders in that binder list between the end
/// of the binder list and the binder this variable originated from.
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
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
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub struct Lvl {
    pub fst: usize,
    pub snd: usize,
}
