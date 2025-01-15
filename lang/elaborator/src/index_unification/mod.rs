//! Index unification for dependent pattern matching
//!
//! To understand what index unification does on a high level, consider the following example:
//!
//! ```pol
//! data Bool { T, F }
//! data Nat { Z, S(x: Nat) }
//!
//! data BoolRep(b: Bool) {
//!     TRep : BoolRep(T),
//!     FRep : BoolRep(F),
//! }
//!
//! def BoolRep(T).foo : Nat {
//!     TRep => 0,
//!     FRep absurd
//! }
//! ```
//!
//! Here, `BoolRep` is an indexed data type that lifts a Boolean value to the type level.
//! The definition `foo` takes a `BoolRep(T)` and returns a `Nat`.
//! The unification algorithm in this module verifies that `TRep` is the only possible constructor based on the type index `b = T`.
//!
//! More generally, given a pattern match, for each clause, the algorithm equates the type indices of the constructor definition with the type indices of the scrutinee.
//! This equation is being decomposed into a set of constraints that are then solved by the unification algorithm.
//! The result of unification is a substitution that equalizes these type indices.
//! The elaborator will apply this substitution to the context, the right-hand side and the type of the clause before proceeding to typecheck the clause.
//!
//! Copattern matching is handled analogously.

pub mod constraints;
pub mod dec;
pub mod unify;
