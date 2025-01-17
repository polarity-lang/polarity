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

use ast::{ctx::values::TypeCtx, Exp, HashMap, MetaVar, MetaVarState};
use codespan::Span;
use constraints::Constraint;
use dec::Dec;
use log::trace;
use printer::Print;
use unify::Ctx;

use crate::result::{TcResult, TypeError};

mod constraints;
mod dec;
mod unify;

pub fn convert(
    ctx: TypeCtx,
    meta_vars: &mut HashMap<MetaVar, MetaVarState>,
    this: Box<Exp>,
    other: &Exp,
    while_elaborating_span: &Option<Span>,
) -> TcResult {
    trace!("{} |- {} =? {}", ctx.print_trace(), this.print_trace(), other.print_trace());
    // Convertibility is checked using the unification algorithm.
    let constraint: Constraint =
        Constraint::Equality { ctx, lhs: this.clone(), rhs: Box::new(other.clone()) };
    let mut ctx = Ctx::new(vec![constraint]);
    match ctx.unify(meta_vars, while_elaborating_span)? {
        Dec::Yes => Ok(()),
        Dec::No => Err(TypeError::not_eq(&this, other, while_elaborating_span)),
    }
}

#[cfg(test)]
mod test {
    use ast::{
        ctx::values::{Binder, TypeCtx},
        HashMap, Idx, MetaVar, MetaVarState, TypeUniv, VarBind, VarBound, Variable,
    };

    use crate::conversion_checking::{constraints::Constraint, dec::Dec, unify::Ctx};

    /// Assert that the two expressions are convertible
    fn check_eq<E: Into<ast::Exp>>(ctx: TypeCtx, e1: E, e2: E) {
        let constraint =
            Constraint::Equality { ctx, lhs: Box::new(e1.into()), rhs: Box::new(e2.into()) };

        let mut ctx = Ctx::new(vec![constraint]);
        let mut hm: HashMap<MetaVar, MetaVarState> = Default::default();
        assert!(ctx.unify(&mut hm, &None).unwrap() == Dec::Yes)
    }

    /// Assert that the two expressions are not convertible
    fn check_neq<E: Into<ast::Exp>>(ctx: TypeCtx, e1: E, e2: E) {
        let constraint =
            Constraint::Equality { ctx, lhs: Box::new(e1.into()), rhs: Box::new(e2.into()) };

        let mut ctx = Ctx::new(vec![constraint]);
        let mut hm: HashMap<MetaVar, MetaVarState> = Default::default();
        assert!(ctx.unify(&mut hm, &None).unwrap() == Dec::No)
    }

    /// Check that `[[a: Type, v: a]] |- v =? v` holds.
    #[test]
    fn convert_var_var_1() {
        let v = Variable {
            span: None,
            idx: Idx { fst: 0, snd: 0 },
            name: VarBound { span: None, id: "x".to_string() },
            inferred_type: None,
        };
        let ctx = vec![vec![
            Binder {
                name: VarBind::Var { span: None, id: "a".to_string() },
                typ: Box::new(TypeUniv { span: None }.into()),
            },
            Binder {
                name: VarBind::Var { span: None, id: "v".to_string() },
                typ: Box::new(
                    Variable {
                        span: None,
                        idx: Idx { fst: 0, snd: 1 },
                        name: VarBound { span: None, id: "a".to_string() },
                        inferred_type: None,
                    }
                    .into(),
                ),
            },
        ]];
        check_eq(ctx.into(), v.clone(), v)
    }

    /// Check that `[[a: Type, v', v]] |- v =? v'` does not hold.
    #[test]
    fn convert_var_var_2() {
        let v1 = Variable {
            span: None,
            idx: Idx { fst: 0, snd: 0 },
            name: VarBound { span: None, id: "v".to_string() },
            inferred_type: None,
        };

        let v2 = Variable {
            span: None,
            idx: Idx { fst: 1, snd: 0 },
            name: VarBound { span: None, id: "v'".to_string() },
            inferred_type: None,
        };

        let ctx = vec![vec![
            Binder {
                name: VarBind::Var { span: None, id: "a".to_string() },
                typ: Box::new(TypeUniv { span: None }.into()),
            },
            Binder {
                name: VarBind::Var { span: None, id: "v'".to_string() },
                typ: Box::new(
                    Variable {
                        span: None,
                        idx: Idx { fst: 0, snd: 2 },
                        name: VarBound { span: None, id: "a".to_string() },
                        inferred_type: None,
                    }
                    .into(),
                ),
            },
            Binder {
                name: VarBind::Var { span: None, id: "v".to_string() },
                typ: Box::new(
                    Variable {
                        span: None,
                        idx: Idx { fst: 0, snd: 2 },
                        name: VarBound { span: None, id: "a".to_string() },
                        inferred_type: None,
                    }
                    .into(),
                ),
            },
        ]];

        check_neq(ctx.into(), v1, v2);
    }

    /// Check that `[] |- Type =? Type` holds.
    #[test]
    fn convert_type_type() {
        let t = TypeUniv { span: None };
        let ctx = vec![];
        check_eq(ctx.into(), t.clone(), t);
    }
}
