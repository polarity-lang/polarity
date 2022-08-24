#[cfg(feature = "use-serde")]
use serde_derive::{Deserialize, Serialize};

use super::common::*;
use super::de_bruijn::*;

#[derive(Clone, Debug)]
#[cfg_attr(feature = "use-serde", derive(Serialize, Deserialize))]
pub enum Var {
    Bound(Idx),
    Free(Ident),
}
