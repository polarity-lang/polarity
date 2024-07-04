use std::fmt;

use derivative::Derivative;

pub type Ident = String;

/// A metavariable which stands for unknown terms which
/// have to be determined during elaboration.
#[derive(Debug, Clone, Copy, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct MetaVar {
    pub id: u64,
}

// Difference between two-level deBruijn indizes and levels
//
// Suppose we have the following context with a variable `v` which
// should point to the element `f` in the context.
//
// ```text
//  [[a,b,c],[d,e],[f,g,h],[i]] ⊢ v
//                  ^             ^
//                  \-------------/
// ```
//
// There are two ways to achieve this. We can either count in the context from the
// right, this is called De Bruijn indices, or we can count from the left, this is called
// De Bruijn levels. Indices look like this:
//
// ```text
//  snd:                2 1 0
//      [[a,b,c],[d,e],[f,g,h],[i]] ⊢ Idx { fst: 1, snd: 2}
//        ^^^^^   ^^^   ^^^^^   ^
//  fst:    3      2      1     0
// ```
// and levels look like this:
// ```text
//  snd:                0 1 2
//      [[a,b,c],[d,e],[f,g,h],[i]] ⊢ Lvl { fst: 2, snd: 0}
//        ^^^^^   ^^^   ^^^^^   ^
//  fst     0      1      2     3
// ```
//
// We use levels when we want to weaken the context, because the binding structure
// remains intact when we add new binders `[j,k,l]` to the right of the context:
// ```text
//  snd:                0 1 2
//      [[a,b,c],[d,e],[f,g,h],[i],[j,k,l]] ⊢ Lvl { fst: 2, snd: 0}
//        ^^^^^   ^^^   ^^^^^   ^   ^^^^^
//  fst     0      1      2     3     4
// ```
// We didn't have to change the level, and it still refers to the same element of the context.

/// Two-dimensional De Bruijn index
///
/// The first component counts the number of binder lists in scope between the variable
/// and the binder list it originated from.
/// The second component counts the number of binders in that binder list between the end
/// of the binder list and the binder this variable originated from.
///
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
