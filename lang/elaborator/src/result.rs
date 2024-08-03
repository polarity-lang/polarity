use miette::{Diagnostic, SourceSpan};
use miette_util::ToMiette;
use thiserror::Error;

use syntax::ast::LookupError;

use codespan::Span;
use std::rc::Rc;
use syntax::ast::*;

use printer::types::Print;

fn comma_separated<I: IntoIterator<Item = String>>(iter: I) -> String {
    separated(", ", iter)
}

fn separated<I: IntoIterator<Item = String>>(s: &str, iter: I) -> String {
    let vec: Vec<_> = iter.into_iter().collect();
    vec.join(s)
}

#[derive(Error, Diagnostic, Debug)]
pub enum TypeError {
    #[error("Wrong number of arguments to {name} provided: got {actual}, expected {expected}")]
    #[diagnostic(code("T-001"))]
    ArgLenMismatch {
        name: String,
        expected: usize,
        actual: usize,
        #[label]
        span: Option<SourceSpan>,
    },
    #[error("The following terms are not equal:\n  1: {lhs}\n  2: {rhs}\n")]
    #[diagnostic(code("T-002"))]
    NotEq {
        lhs: String,
        rhs: String,
        #[label("Source of (1)")]
        lhs_span: Option<SourceSpan>,
        #[label("Source of (2)")]
        rhs_span: Option<SourceSpan>,
    },
    #[error("Cannot match on codata type {name}")]
    #[diagnostic(code("T-003"))]
    MatchOnCodata {
        name: Ident,
        #[label]
        span: Option<SourceSpan>,
    },
    #[error("Cannot comatch on data type {name}")]
    #[diagnostic(code("T-004"))]
    ComatchOnData {
        name: Ident,
        #[label]
        span: Option<SourceSpan>,
    },
    #[error("Invalid pattern match: {msg}")]
    #[diagnostic(code("T-005"))]
    InvalidMatch {
        msg: String,
        #[label]
        span: Option<SourceSpan>,
    },
    #[error("Got {actual}, which is not in type {expected}")]
    #[diagnostic(code("T-006"))]
    NotInType {
        expected: Ident,
        actual: Ident,
        #[label]
        span: Option<SourceSpan>,
    },
    #[error("Pattern for {name} is marked as absurd but that could not be proven")]
    #[diagnostic(code("T-007"))]
    PatternIsNotAbsurd {
        name: Ident,
        #[label]
        span: Option<SourceSpan>,
    },
    #[error("Pattern for {name} is absurd and must be marked accordingly")]
    #[diagnostic(code("T-008"))]
    PatternIsAbsurd {
        name: Ident,
        #[label]
        span: Option<SourceSpan>,
    },
    #[error("Type annotation required for match expression")]
    #[diagnostic(code("T-009"))]
    CannotInferMatch {
        #[label]
        span: Option<SourceSpan>,
    },
    #[error("Type annotation required for comatch expression")]
    #[diagnostic(code("T-010"))]
    CannotInferComatch {
        #[label]
        span: Option<SourceSpan>,
    },
    #[error("Type annotation required for typed hole")]
    #[diagnostic(code("T-011"))]
    CannotInferHole {
        #[label]
        span: Option<SourceSpan>,
    },
    #[error("Expected type constructor application, got {got}")]
    #[diagnostic(code("T-012"))]
    ExpectedTypApp {
        got: String,
        #[label]
        span: Option<SourceSpan>,
    },
    #[error("Local comatch not supported for type {type_name} because {type_name} contains destructors with self parameters")]
    #[diagnostic(code("T-013"), help("Use a top-level codefinition instead"))]
    LocalComatchWithSelf {
        type_name: String,
        #[label]
        span: Option<SourceSpan>,
    },
    #[error("{idx} occurs in {exp}")]
    #[diagnostic(code("T-014"))]
    OccursCheckFailed {
        idx: Idx,
        exp: String,
        #[label]
        span: Option<SourceSpan>,
    },
    #[error("Cannot unify annotated expression {exp}")]
    #[diagnostic(code("T-015"))]
    UnsupportedAnnotation {
        exp: String,
        #[label]
        span: Option<SourceSpan>,
    },
    #[error("Cannot automatically decide whether {lhs} and {rhs} unify")]
    #[diagnostic(code("T-016"))]
    CannotDecide {
        lhs: String,
        rhs: String,
        #[label]
        lhs_span: Option<SourceSpan>,
        #[label]
        rhs_span: Option<SourceSpan>,
    },
    #[error("An unexpected internal error occurred: {message}")]
    #[diagnostic(code("T-XXX"))]
    /// This error should not occur.
    /// Some internal invariant has been violated.
    Impossible {
        message: String,
        #[label]
        span: Option<SourceSpan>,
    },
    #[error(transparent)]
    #[diagnostic(transparent)]
    Lookup(#[from] LookupError),
}

impl TypeError {
    pub fn not_eq(lhs: Rc<Exp>, rhs: Rc<Exp>) -> Self {
        Self::NotEq {
            lhs: lhs.print_to_string(None),
            rhs: rhs.print_to_string(None),
            lhs_span: lhs.span().to_miette(),
            rhs_span: rhs.span().to_miette(),
        }
    }

    pub fn invalid_match(
        missing: HashSet<String>,
        undeclared: HashSet<String>,
        duplicate: HashSet<String>,
        info: &Option<Span>,
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

        Self::InvalidMatch { msg: separated("; ", msgs), span: info.to_miette() }
    }

    pub fn expected_typ_app(got: Rc<Exp>) -> Self {
        Self::ExpectedTypApp { got: got.print_to_string(None), span: got.span().to_miette() }
    }

    pub fn occurs_check_failed(idx: Idx, exp: Rc<Exp>) -> Self {
        Self::OccursCheckFailed {
            idx,
            exp: exp.print_to_string(None),
            span: exp.span().to_miette(),
        }
    }

    pub fn unsupported_annotation(exp: Rc<Exp>) -> Self {
        Self::UnsupportedAnnotation { exp: exp.print_to_string(None), span: exp.span().to_miette() }
    }

    pub fn cannot_decide(lhs: Rc<Exp>, rhs: Rc<Exp>) -> Self {
        Self::CannotDecide {
            lhs: lhs.print_to_string(None),
            rhs: rhs.print_to_string(None),
            lhs_span: lhs.span().to_miette(),
            rhs_span: rhs.span().to_miette(),
        }
    }
}
