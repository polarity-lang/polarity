//! Unification for conversion checking
//!
//! Conversion checking is the process of checking whether two types are equal.
//! It is used in various places of the typechecker.
//! For instance, when checking a type annotation `(e: t)` against an expected type `s`,
//! we need to check that `t` is equal to `s`, i.e. that `t` is convertible to `s`.
//!
//! To do so, we use a unification algorithm that decomposes the equality between `t` and `s` into a set of constraints.
//! Conversion checking succeeds if all constraints can be solved by the unification algorithm.
//!
//! While solving constraints, the unification algorithm may encounter holes in the terms that are being equated.
//! Each hole is identified by a unique metavariable.
//! During unification, it is possible that we find out that a metavariable must be equal to another term which - under certain conditions - may solve the metavariable globally.
//! This means that all other occurrences of the metavariable can be solved by the same term.
//!
//! This process of solving metavariables is especially relevant to figure out the solution to implicit arguments.
//! For instance, consider the following example:
//!
//! ```pol
//! -- | The type of non-dependent functions.
//! codata Fun(a b: Type) {
//!     -- | Application of a function to its argument.
//!     Fun(a, b).ap(implicit a b: Type, x: a): b
//! }
//!
//! data Nat {
//!     Z,
//!     S(n: Nat)
//! }
//!
//! let main: Nat {
//!     (\x. x).ap(0)
//! }
//! ```
//!
//! In this example, we do not need to provide the types `a` and `b` explicitly when calling `ap`.
//! Instead, the lowering phase will insert holes for these arguments as follows:
//! ```pol
//! let main: Nat {
//!    (\x. x).ap(?a, ?b, 0)
//! }
//! ```
//! Typechecking the arguments to `ap` invokes conversion checking and unifies `?a` with `Nat`.
//! Inferring the type of the lambda expression `(\x. x)` will equate the expected return type `?b` with the inferred return type of `x: Nat`.
//! This will solve the metavariable `?b` with `Nat`.
//!
//! You can retrace this result by explicitly providing holes to the implicit arguments like so:
//! ```pol
//! let main: Nat {
//!     (\x. x).ap(a:=_, b:=_, 0)
//! }
//! ```
//! When hovering over the holes in an editor connected to our language server, you will see that both holes are solved with `Nat`.

pub mod constraints;
pub mod dec;
pub mod unify;
