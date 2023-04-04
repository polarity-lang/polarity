use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

use syntax::generic::LookupError;

#[derive(Error, Diagnostic, Debug)]
pub enum EvalError {
    #[error(transparent)]
    #[diagnostic(transparent)]
    Lookup(#[from] LookupError),
    #[error("The impossible happened: {message}")]
    #[diagnostic(code("E-XXX"))]
    /// This error should not occur.
    /// Some internal invariant has been violated.
    Impossible {
        message: String,
        #[label]
        span: Option<SourceSpan>,
    },
}
