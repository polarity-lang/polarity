use std::rc::Rc;

use miette::{Diagnostic, SourceSpan};
use miette_util::ToMiette;
use thiserror::Error;

use data::string::{comma_separated, separated};
use data::HashSet;
use syntax::common::*;
use syntax::nf;
use syntax::ust;

use printer::PrintToString;

use super::unify::UnifyError;

#[derive(Error, Diagnostic, Debug)]
pub enum TypeError {
    // TODO: Add span
    #[diagnostic()]
    #[error("Wrong number of arguments to {name} provided: got {actual}, expected {expected}")]
    ArgLenMismatch {
        name: String,
        expected: usize,
        actual: usize,
        #[label]
        span: Option<SourceSpan>,
    },
    #[diagnostic()]
    #[error("{lhs} is not equal to {rhs}")]
    NotEq {
        lhs: String,
        rhs: String,
        #[label]
        lhs_span: Option<SourceSpan>,
        #[label]
        rhs_span: Option<SourceSpan>,
    },
    #[diagnostic()]
    #[error("Cannot match on codata type {name}")]
    MatchOnCodata { name: Ident, span: Option<SourceSpan> },
    #[diagnostic()]
    #[error("Cannot comatch on data type {name}")]
    ComatchOnData { name: Ident, span: Option<SourceSpan> },
    #[diagnostic()]
    #[error("Invalid pattern match: {msg}")]
    InvalidMatch {
        msg: String,
        #[label]
        span: Option<SourceSpan>,
    },
    #[diagnostic()]
    #[error("Got {actual}, which is not in type {expected}")]
    NotInType {
        expected: Ident,
        actual: Ident,
        #[label]
        span: Option<SourceSpan>,
    },
    #[diagnostic()]
    #[error("Pattern for {name} is marked as absurd but that could not be proven")]
    PatternIsNotAbsurd {
        name: Ident,
        #[label]
        span: Option<SourceSpan>,
    },
    #[diagnostic()]
    #[error("Pattern for {name} is absurd and must be marked accordingly")]
    PatternIsAbsurd {
        name: Ident,
        #[label]
        span: Option<SourceSpan>,
    },
    #[diagnostic()]
    #[error("Type annotation required")]
    AnnotationRequired {
        #[label]
        span: Option<SourceSpan>,
    },
    #[diagnostic()]
    #[error("Expected type constructor application, got {got}")]
    ExpectedTypApp {
        got: String,
        #[label]
        span: Option<SourceSpan>,
    },
    #[diagnostic()]
    #[error("The impossible happened: {message}")]
    /// This error should not occur.
    /// Some internal invariant has been violated.
    Impossible { message: String, span: Option<SourceSpan> },
    // TODO: Add span
    #[diagnostic(transparent)]
    #[error(transparent)]
    Unify(#[from] UnifyError),
    #[diagnostic(transparent)]
    #[error(transparent)]
    Normalize(#[from] NormalizeError),
}

impl TypeError {
    pub fn not_eq(lhs: Rc<nf::Nf>, rhs: Rc<nf::Nf>) -> Self {
        Self::NotEq {
            lhs: lhs.print_to_string(),
            rhs: rhs.print_to_string(),
            lhs_span: lhs.info().span.to_miette(),
            rhs_span: rhs.info().span.to_miette(),
        }
    }

    pub fn invalid_match(
        missing: HashSet<String>,
        undeclared: HashSet<String>,
        duplicate: HashSet<String>,
        info: &ust::Info,
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

        Self::InvalidMatch { msg: separated("; ", msgs), span: info.span.to_miette() }
    }

    pub fn expected_typ_app(got: Rc<nf::Nf>) -> Self {
        Self::ExpectedTypApp { got: got.print_to_string(), span: got.info().span.to_miette() }
    }
}

impl From<EvalError> for TypeError {
    fn from(err: EvalError) -> Self {
        Self::Normalize(err.into())
    }
}

impl From<ReadBackError> for TypeError {
    fn from(err: ReadBackError) -> Self {
        Self::Normalize(err.into())
    }
}

#[derive(Error, Diagnostic, Debug)]
#[diagnostic(transparent)]
#[error(transparent)]
pub enum NormalizeError {
    Eval(#[from] EvalError),
    ReadBack(#[from] ReadBackError),
}

#[derive(Error, Diagnostic, Debug)]
#[diagnostic()]
pub enum EvalError {
    #[error("Trying to evaluate hole of type {typ} {span:?}")]
    EvalHole {
        typ: String,
        #[label]
        span: Option<SourceSpan>,
    },
    #[error("The impossible happened: {message}")]
    /// This error should not occur.
    /// Some internal invariant has been violated.
    Impossible {
        message: String,
        #[label]
        span: Option<SourceSpan>,
    },
}

#[derive(Error, Diagnostic, Debug)]
#[diagnostic(transparent)]
#[error(transparent)]
pub enum ReadBackError {
    Eval(#[from] EvalError),
}
