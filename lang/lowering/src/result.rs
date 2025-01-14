use miette::{Diagnostic, SourceSpan};
use parser::cst::ident::Ident;
use thiserror::Error;

#[derive(Error, Diagnostic, Debug, Clone)]
pub enum LoweringError {
    #[error("Undefined identifier {}", name.id)]
    #[diagnostic(code("L-001"))]
    UndefinedIdent {
        name: Ident,
        #[label]
        span: SourceSpan,
    },
    #[error("Duplicate definition of {}", name.id)]
    #[diagnostic(code("L-002"))]
    AlreadyDefined {
        name: Ident,
        #[label]
        span: SourceSpan,
    },
    #[error("{} must be used as destructor", name.id)]
    #[diagnostic(code("L-003"))]
    MustUseAsDotCall {
        name: Ident,
        #[label]
        span: SourceSpan,
    },
    #[error("{} cannot be used as a destructor", name.id)]
    #[diagnostic(code("L-004"))]
    CannotUseAsDotCall {
        name: Ident,
        #[label]
        span: SourceSpan,
    },
    #[error("Arguments to type constructor {} must be provided for {}", typ.id, xtor.id)]
    #[diagnostic(code("L-005"))]
    MustProvideArgs {
        xtor: Ident,
        typ: Ident,
        #[label]
        span: SourceSpan,
    },
    #[error("The annotated label {name} is shadowed by a local variable")]
    #[diagnostic(code("L-007"))]
    LabelShadowed {
        name: String,
        #[label]
        span: SourceSpan,
    },
    #[error("The annotated label {name} is not unique")]
    #[diagnostic(code("L-008"))]
    LabelNotUnique {
        name: String,
        #[label]
        span: SourceSpan,
    },
    #[error("Expected a type constructor")]
    #[diagnostic(code("L-009"))]
    ExpectedTypCtor {
        #[label]
        span: SourceSpan,
    },
    #[error("Literal cannot be desugared because S/Z are not in program")]
    #[diagnostic(code("L-010"))]
    NatLiteralCannotBeDesugared {
        #[label]
        span: SourceSpan,
    },
    #[error("Mismatched named arguments: given {}, expected {}", given.id, expected.id)]
    #[diagnostic(code("L-011"))]
    MismatchedNamedArgs {
        given: Ident,
        expected: Ident,
        #[label]
        span: SourceSpan,
    },
    #[error("Used named argument {} for wildcard parameter", given.id)]
    #[diagnostic(code("L-012"))]
    NamedArgForWildcard {
        given: Ident,
        #[label]
        span: SourceSpan,
    },
    #[error("Missing argument for parameter {}", expected.id)]
    #[diagnostic(code("L-013"))]
    MissingArgForParam {
        expected: Ident,
        #[label]
        span: SourceSpan,
    },
    #[error("Too many arguments provided")]
    #[diagnostic(code("L-014"))]
    TooManyArgs {
        #[label]
        span: SourceSpan,
    },
    #[error("Type universe \"Type\" does not take arguments")]
    #[diagnostic(code("L-015"))]
    TypeUnivArgs {
        #[label]
        span: SourceSpan,
    },
    #[error("\"Type\" is not a valid identifier")]
    #[diagnostic(code("L-016"))]
    TypeUnivIdentifier {
        #[label]
        span: SourceSpan,
    },
    #[error("An unexpected internal error occurred: {message}")]
    #[diagnostic(code("L-XXX"))]
    /// This error should not occur.
    /// Some internal invariant has been violated.
    Impossible {
        message: String,
        #[label]
        span: Option<SourceSpan>,
    },
}
