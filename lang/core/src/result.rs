use std::error::Error;
use std::fmt;
use std::rc::Rc;

use data::HashSet;
use syntax::common::*;
use syntax::ust;

use printer::PrintToString;

use super::unify::UnifyError;

#[derive(Debug)]
pub enum TypeError {
    ArgLenMismatch { expected: usize, actual: usize },
    NotEq { lhs: Rc<ust::Exp>, rhs: Rc<ust::Exp> },
    InvalidMatch { missing: HashSet<Ident>, undeclared: HashSet<Ident>, duplicate: HashSet<Ident> },
    NotInType { expected: Ident, actual: Ident },
    PatternIsNotAbsurd { name: Ident },
    PatternIsAbsurd { name: Ident },
    Unify(UnifyError),
}

impl Error for TypeError {}

impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeError::ArgLenMismatch { expected, actual } => write!(
                f,
                "Wrong number of arguments provided: got {}, expected {}",
                actual, expected
            ),
            TypeError::NotEq { lhs, rhs } => {
                write!(f, "{} is not equal to {}", lhs.print_to_string(), rhs.print_to_string())
            }
            TypeError::InvalidMatch { missing, undeclared, duplicate } => {
                write!(f, "Invalid pattern match: ")?;

                let mut msgs = Vec::new();

                if !missing.is_empty() {
                    msgs.push(format!("missing {}", comma_separated(missing.iter().cloned())));
                }
                if !undeclared.is_empty() {
                    msgs.push(format!(
                        "undeclared {}",
                        comma_separated(undeclared.iter().cloned())
                    ));
                }
                if !duplicate.is_empty() {
                    msgs.push(format!("duplicate {}", comma_separated(duplicate.iter().cloned())));
                }

                write!(f, "{}", separated("; ", msgs))?;

                Ok(())
            }
            TypeError::NotInType { expected, actual } => {
                write!(f, "Got {}, which is not in type {}", actual, expected)
            }
            TypeError::PatternIsNotAbsurd { name } => {
                write!(f, "Pattern for {} is marked as absurd but that could not be proven", name)
            }
            TypeError::PatternIsAbsurd { name } => {
                write!(f, "Pattern for {} is absurd and must be marked accordingly", name)
            }
            TypeError::Unify(err) => err.fmt(f),
        }
    }
}

fn comma_separated<I: IntoIterator<Item = String>>(iter: I) -> String {
    separated(", ", iter)
}

fn separated<I: IntoIterator<Item = String>>(s: &str, iter: I) -> String {
    let vec: Vec<_> = iter.into_iter().collect();
    vec.join(s)
}
