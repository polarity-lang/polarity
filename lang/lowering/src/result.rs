use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

use syntax::common::*;

#[derive(Error, Diagnostic, Debug)]
pub enum LoweringError {
    #[error("Undefined identifier {name}")]
    #[diagnostic(code("L-001"))]
    UndefinedIdent {
        name: Ident,
        #[label]
        span: SourceSpan,
    },
    #[error("Duplicate definition of {name}")]
    #[diagnostic(code("L-002"))]
    AlreadyDefined {
        name: Ident,
        #[label]
        span: Option<SourceSpan>,
    },
    #[error("{name} must be used as destructor")]
    #[diagnostic(code("L-003"))]
    MustUseAsDtor {
        name: Ident,
        #[label]
        span: SourceSpan,
    },
    #[error("{name} cannot be used as a destructor")]
    #[diagnostic(code("L-004"))]
    CannotUseAsDtor {
        name: Ident,
        #[label]
        span: SourceSpan,
    },
    #[error("Arguments to type constructor {typ} must be provided for {xtor}")]
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
}
