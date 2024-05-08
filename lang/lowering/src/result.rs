use miette::{Diagnostic, SourceSpan};
use parser::cst::ident::Ident;
use syntax::ast::lookup_table::DeclKind;
use thiserror::Error;

#[derive(Error, Diagnostic, Debug)]
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
        span: Option<SourceSpan>,
    },
    #[error("{} must be used as destructor", name.id)]
    #[diagnostic(code("L-003"))]
    MustUseAsDtor {
        name: Ident,
        #[label]
        span: SourceSpan,
    },
    #[error("{} cannot be used as a destructor", name.id)]
    #[diagnostic(code("L-004"))]
    CannotUseAsDtor {
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
    #[error("Expected {name} to be a {expected}, but it is a {actual}")]
    #[diagnostic(code("L-006"))]
    InvalidDeclarationKind { name: String, expected: DeclKind, actual: DeclKind },
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
