use std::rc::Rc;

use miette::Diagnostic;
use thiserror::Error;

use data::string::{comma_separated, separated};
use data::HashSet;
use syntax::common::*;
use syntax::ust;

use printer::PrintToString;

use super::unify::UnifyError;

#[derive(Error, Diagnostic, Debug)]
pub enum TypeError {
    #[error("Wrong number of arguments provided: got {actual}, expected {expected}")]
    ArgLenMismatch { expected: usize, actual: usize },
    #[error("{lhs} is not equal to {rhs}")]
    NotEq { lhs: String, rhs: String },
    #[error("Invalid pattern match: {msg}")]
    InvalidMatch { msg: String },
    #[error("Got {actual}, which is not in type {expected}")]
    NotInType { expected: Ident, actual: Ident },
    #[error("Pattern for {name} is marked as absurd but that could not be proven")]
    PatternIsNotAbsurd { name: Ident },
    #[error("Pattern for {name} is absurd and must be marked accordingly")]
    PatternIsAbsurd { name: Ident },
    #[error(transparent)]
    Unify(#[from] UnifyError),
}

impl TypeError {
    pub fn not_eq(lhs: Rc<ust::Exp>, rhs: Rc<ust::Exp>) -> Self {
        Self::NotEq { lhs: lhs.print_to_string(), rhs: rhs.print_to_string() }
    }

    pub fn invalid_match(
        missing: HashSet<String>,
        undeclared: HashSet<String>,
        duplicate: HashSet<String>,
    ) -> Self {
        let mut msgs = Vec::new();

        if !missing.is_empty() {
            msgs.push(format!("missing {}", comma_separated(missing.iter().cloned())));
        }
        if !undeclared.is_empty() {
            msgs.push(format!("undeclared {}", comma_separated(undeclared.iter().cloned())));
        }
        if !duplicate.is_empty() {
            msgs.push(format!("duplicate {}", comma_separated(duplicate.iter().cloned())));
        }

        Self::InvalidMatch { msg: separated("; ", msgs) }
    }
}
