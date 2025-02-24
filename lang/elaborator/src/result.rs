use std::convert::Infallible;

use miette::{Diagnostic, SourceSpan};
use miette_util::codespan::Span;
use miette_util::ToMiette;
use thiserror::Error;

use ast::*;
use printer::types::Print;

fn comma_separated<I: IntoIterator<Item = String>>(iter: I) -> String {
    separated(", ", iter)
}

fn separated<I: IntoIterator<Item = String>>(s: &str, iter: I) -> String {
    let vec: Vec<_> = iter.into_iter().collect();
    vec.join(s)
}

/// The result type specialized to type errors.
pub type TcResult<T = ()> = Result<T, Box<TypeError>>;

/// This enum contains all errors that can be emitted during elaboration, i.e. either
/// during bidirectional type inference, normalization, index unification or conversion checking.
#[derive(Error, Diagnostic, Debug, Clone, PartialEq, Eq)]
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
    /// This is one of three [TypeError] variants that are emitted when we perform conversion checking
    /// and the expressions are not convertible. The variants [TypeError::NotEq], [TypeError::NotEqDetailed]
    /// and [TypeError::NotEqInternal] exist in order to improve the quality of the error messages.
    ///
    /// - The variant [TypeError::NotEq] is used for expressions which are not equal, and when we can also see
    ///   this from the outermost constructor. For example, `Bool` and `Nat`.
    /// - The variant [TypeError::NotEqDetailed] is used for expressions which are not equal, but which have an
    ///   outermost constructor in common. An example of this is `S(x)` and `S(y)` or `List(Nat)` and `List(Bool)`.
    ///   In that case we have additional fields [TypeError::NotEqDetailed::lhs_internal] and
    ///   [TypeError::NotEqDetailed::rhs_internal] which store the String representation of the two subexpressions
    ///   which differ: `x` and `y`, resp. `Nat` and `Bool` in this example.
    /// - The variant [TypeError::NotEqInternal] should never be presented to the user. It is only used internally in
    ///   the implementation of conversion checking. If we check whether `List(Int)` and `List(Nat)` are convertible, then
    ///   we first internally throw a [TypeError::NotEqInternal] which carries the information that `Int` and `Nat`
    ///   are not equal. We then catch this error and insert the information in the [TypeError::NotEqDetailed] variant.
    ///
    #[error("The following terms are not equal:\n  1: {lhs}\n  2: {rhs}\n")]
    #[diagnostic(code("T-002"))]
    NotEq {
        lhs: String,
        rhs: String,
        #[label("Source of (1)")]
        lhs_span: Option<SourceSpan>,
        #[label("Source of (2)")]
        rhs_span: Option<SourceSpan>,
        #[label("While elaborating")]
        while_elaborating_span: Option<SourceSpan>,
    },
    /// See documentation of [TypeError::NotEq] for details.
    #[error("The following terms are not equal:\n  1: {lhs}\n  2: {rhs}\n")]
    #[diagnostic(
        code("T-002"),
        help("The two subterms {lhs_internal} and {rhs_internal} are not equal.")
    )]
    NotEqDetailed {
        lhs: String,
        rhs: String,
        #[label("Source of (1)")]
        lhs_span: Option<SourceSpan>,
        #[label("Source of (2)")]
        rhs_span: Option<SourceSpan>,
        lhs_internal: String,
        rhs_internal: String,
        #[label("While elaborating")]
        while_elaborating_span: Option<SourceSpan>,
    },
    /// See documentation of [TypeError::NotEq] for details.
    #[error("INTERNAL:\n  1: {lhs}\n  2: {rhs}\n")]
    NotEqInternal { lhs: String, rhs: String },
    #[error("Cannot match on codata type {name}")]
    #[diagnostic(code("T-003"))]
    MatchOnCodata {
        name: Box<IdBound>,
        #[label]
        span: Option<SourceSpan>,
    },
    #[error("Cannot comatch on data type {name}")]
    #[diagnostic(code("T-004"))]
    ComatchOnData {
        name: Box<IdBound>,
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
        expected: Box<IdBind>,
        actual: Box<IdBound>,
        #[label]
        span: Option<SourceSpan>,
    },
    #[error("Pattern for {name} is marked as absurd but that could not be proven")]
    #[diagnostic(code("T-007"))]
    PatternIsNotAbsurd {
        name: Box<IdBound>,
        #[label]
        span: Option<SourceSpan>,
    },
    #[error("Pattern for {name} is absurd and must be marked accordingly")]
    #[diagnostic(code("T-008"))]
    PatternIsAbsurd {
        name: Box<IdBound>,
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
    #[error("Cannot automatically decide whether {lhs} and {rhs} unify")]
    #[diagnostic(code("T-016"))]
    CannotDecide {
        lhs: String,
        rhs: String,
        #[label]
        lhs_span: Option<SourceSpan>,
        #[label]
        rhs_span: Option<SourceSpan>,
        #[label("While elaborating")]
        while_elaborating_span: Option<SourceSpan>,
    },
    #[error("The metavariable {meta_var} could not be solved")]
    #[diagnostic(code("T-017"))]
    UnresolvedMeta {
        #[label]
        span: Option<SourceSpan>,
        meta_var: String,
    },
    #[error("A case for constructor {name} was missing during evaluation.")]
    #[diagnostic(code("T-018"))]
    MissingCase { name: String },
    #[error("A case for destructor {name} was missing during evaluation.")]
    #[diagnostic(code("T-019"))]
    MissingCocase { name: String },
    #[error("The metavariable {meta_var} received {arg} as an argument more than once")]
    #[diagnostic(
        code("T-020"),
        help("This means that the metavariable cannot be solved automatically.")
    )]
    MetaArgNotDistinct {
        #[label]
        span: Option<SourceSpan>,
        meta_var: String,
        arg: String,
        #[label("While elaborating")]
        while_elaborating_span: Option<SourceSpan>,
    },
    #[error("The metavariable {meta_var} received argument {arg} which is not a variable")]
    #[diagnostic(
        code("T-021"),
        help("This means that the metavariable cannot be solved automatically.")
    )]
    MetaArgNotVariable {
        #[label]
        span: Option<SourceSpan>,
        meta_var: String,
        arg: String,
        #[label("While elaborating")]
        while_elaborating_span: Option<SourceSpan>,
    },
    #[error("The metavariable {meta_var} was equated with an expression that contains {out_of_scope} which is not in scope for {meta_var}")]
    #[diagnostic(
        code("T-022"),
        help("This means that the metavariable cannot be solved automatically.")
    )]
    MetaEquatedToOutOfScope {
        #[label]
        span: Option<SourceSpan>,
        meta_var: String,
        out_of_scope: String,
        #[label("While elaborating")]
        while_elaborating_span: Option<SourceSpan>,
    },
    #[error("The metavariable {meta_var} was equated with an expression that itself contains {meta_var}")]
    #[diagnostic(
        code("T-023"),
        help("This means that the metavariable cannot be solved automatically.")
    )]
    MetaOccursCheckFailed {
        #[label]
        span: Option<SourceSpan>,
        meta_var: String,
        #[label("While elaborating")]
        while_elaborating_span: Option<SourceSpan>,
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
}

impl TypeError {
    pub fn invalid_match(
        missing: HashSet<String>,
        undeclared: HashSet<String>,
        duplicate: HashSet<String>,
        info: &Option<Span>,
    ) -> Box<Self> {
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

        Self::InvalidMatch { msg: separated("; ", msgs), span: info.to_miette() }.into()
    }

    pub fn expected_typ_app(got: &Exp) -> Self {
        Self::ExpectedTypApp { got: got.print_to_string(None), span: got.span().to_miette() }
    }

    pub fn occurs_check_failed(idx: Idx, exp: &Exp) -> Box<Self> {
        Self::OccursCheckFailed {
            idx,
            exp: exp.print_to_string(None),
            span: exp.span().to_miette(),
        }
        .into()
    }

    pub fn cannot_decide(lhs: &Exp, rhs: &Exp, while_elaborating_span: &Option<Span>) -> Box<Self> {
        Self::CannotDecide {
            lhs: lhs.print_to_string(None),
            rhs: rhs.print_to_string(None),
            lhs_span: lhs.span().to_miette(),
            rhs_span: rhs.span().to_miette(),
            while_elaborating_span: while_elaborating_span.to_miette(),
        }
        .into()
    }
}

impl From<Infallible> for TypeError {
    fn from(inf: Infallible) -> Self {
        match inf {}
    }
}

impl From<Infallible> for Box<TypeError> {
    fn from(inf: Infallible) -> Self {
        Box::new(TypeError::from(inf))
    }
}
