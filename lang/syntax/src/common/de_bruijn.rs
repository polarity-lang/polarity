use std::fmt;

/// Two-dimensional De-Bruijn index
///
/// The first component counts the number of binder lists in scope between the variable
/// and the binder list it originated from.
/// The second component counts the number of binders in that binder list between the end
/// of the binder list and the binder this variable originated from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Idx {
    pub fst: usize,
    pub snd: usize,
}

impl fmt::Display for Idx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.fst, self.snd)
    }
}

/// Two-dimensional De-Bruijn level
///
/// The first component counts the number of binder lists in scope between the root of the
/// term and the binder list it originated from.
/// The second component counts the number of binders in that binder list between the start
/// of the binder list and the binder this variable originated from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Lvl {
    pub fst: usize,
    pub snd: usize,
}

impl Lvl {
    pub fn here() -> Self {
        Self { fst: 0, snd: 0 }
    }
}

impl fmt::Display for Lvl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.fst, self.snd)
    }
}

/// Either a De-Bruijn level or an index
///
/// Used to support lookup with both representations using the same interface
#[derive(Debug, Clone, Copy)]
pub enum Var {
    Lvl(Lvl),
    Idx(Idx),
}

impl From<Idx> for Var {
    fn from(idx: Idx) -> Self {
        Var::Idx(idx)
    }
}

impl From<Lvl> for Var {
    fn from(lvl: Lvl) -> Self {
        Var::Lvl(lvl)
    }
}

impl fmt::Display for Var {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Var::Lvl(lvl) => write!(f, "lvl:{lvl}"),
            Var::Idx(idx) => write!(f, "idx:{idx}"),
        }
    }
}
