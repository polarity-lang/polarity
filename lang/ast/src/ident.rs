use std::fmt;

use derivative::Derivative;
use miette_util::codespan::Span;
use pretty::DocAllocator;
use printer::{
    Alloc, Builder, Print, PrintCfg,
    tokens::{AT, DOT, QUESTION_MARK, UNDERSCORE},
};
use url::Url;

use crate::HasSpan;

// Local variables (binding site)
//
//

/// A local variable binding
///
/// E.g. on the left-hand side of a pattern or in a parameter list
#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub enum VarBind {
    Var {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        span: Option<Span>,
        id: String,
    },
    Wildcard {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        span: Option<Span>,
    },
}

impl VarBind {
    pub fn from_string(id: &str) -> Self {
        VarBind::Var { span: None, id: id.to_owned() }
    }
}

impl fmt::Display for VarBind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VarBind::Var { id, .. } => write!(f, "{id}"),
            VarBind::Wildcard { .. } => write!(f, "_"),
        }
    }
}

impl Print for VarBind {
    fn print<'a>(&'a self, _cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        match self {
            VarBind::Var { id, .. } => alloc.text(id),
            VarBind::Wildcard { .. } => alloc.text(UNDERSCORE),
        }
    }
}

impl HasSpan for VarBind {
    fn span(&self) -> Option<Span> {
        match self {
            VarBind::Var { span, .. } => *span,
            VarBind::Wildcard { span } => *span,
        }
    }
}

// Local variables (bound occurence)
//
//

/// A bound occurence of a local variable
#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct VarBound {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub span: Option<Span>,
    pub id: String,
}

impl VarBound {
    pub fn from_string(id: &str) -> Self {
        VarBound { span: None, id: id.to_owned() }
    }
}

impl fmt::Display for VarBound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl HasSpan for VarBound {
    fn span(&self) -> Option<Span> {
        self.span
    }
}

impl Print for VarBound {
    fn print<'a>(&'a self, _cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        alloc.text(self.id.clone())
    }
}

// Global identifiers (binding site)
//
//

/// A global identifier binding
///
/// E.g. the names for (co)data type declarations, (co)def declarations, and top-level let bindings
#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct IdBind {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub span: Option<Span>,
    pub id: String,
}

impl IdBind {
    pub fn from_string(id: &str) -> Self {
        IdBind { span: None, id: id.to_owned() }
    }
}

impl fmt::Display for IdBind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl HasSpan for IdBind {
    fn span(&self) -> Option<Span> {
        self.span
    }
}

impl From<IdBound> for IdBind {
    fn from(id: IdBound) -> Self {
        IdBind { span: id.span, id: id.id }
    }
}

impl PartialEq<IdBound> for IdBind {
    fn eq(&self, other: &IdBound) -> bool {
        self.id == other.id
    }
}

impl PartialEq<IdBind> for IdBound {
    fn eq(&self, other: &IdBind) -> bool {
        self.id == other.id
    }
}

// Global identifiers (bound occurence)
//
//

/// A bound occurence of a global identifier
///
/// E.g. the name in a (type) constructor or destructor call, or in a call to a top-level let binding
#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct IdBound {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub span: Option<Span>,
    pub id: String,
    /// The URI of the module where the identifier was defined
    pub uri: Url,
}

impl fmt::Display for IdBound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl HasSpan for IdBound {
    fn span(&self) -> Option<Span> {
        self.span
    }
}

/// Whether the metavariable corresponds to a typed hole written by the user
/// or whether it was inserted during lowering for an implicit argument.
#[derive(Debug, Clone, Copy, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub enum MetaVarKind {
    /// A typed hole written `_` that must be solved during type inference.
    /// If type inference doesn't find a unique solution, an error is thrown.
    MustSolve,
    /// A typed hole written `?` that stands for an incomplete program.
    /// This hole can be solved during type checking, but we do not throw an error
    /// if it isn't solved.
    CanSolve,
    /// A metavariable which was inserted during lowering for an implicit argument.
    Inserted,
}

/// A metavariable which stands for unknown terms which
/// have to be determined during elaboration.
#[derive(Debug, Clone, Copy, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct MetaVar {
    pub span: Option<Span>,
    pub kind: MetaVarKind,
    pub id: u64,
}

impl MetaVar {
    /// Check whether this metavariable was inserted during lowering for
    /// an implicit argument.
    pub fn is_inserted(&self) -> bool {
        self.kind == MetaVarKind::Inserted
    }

    /// Check whether this metavariable corresponds to a typed hole written
    /// by the programmer.
    pub fn is_user(&self) -> bool {
        self.kind == MetaVarKind::MustSolve || self.kind == MetaVarKind::CanSolve
    }

    /// Metavariables which must be solved during type inference.
    pub fn must_be_solved(&self) -> bool {
        match self.kind {
            MetaVarKind::MustSolve => true,
            MetaVarKind::CanSolve => false,
            MetaVarKind::Inserted => true,
        }
    }
}

impl Print for MetaVar {
    fn print<'a>(&'a self, _cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let MetaVar { kind, id, span: _ } = self;
        let id = alloc.text(format!("{id}"));
        match kind {
            MetaVarKind::MustSolve => alloc.text(UNDERSCORE).append(id),
            MetaVarKind::CanSolve => alloc.text(QUESTION_MARK).append(id),
            MetaVarKind::Inserted => alloc.text("<Inserted>").append(id),
        }
    }
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

impl Print for Idx {
    fn print<'a>(&'a self, _cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Idx { fst, snd } = self;
        alloc.text(AT).append(format!("{fst}")).append(DOT).append(format!("{snd}"))
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
