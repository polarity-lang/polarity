// FIXME: Ignore lints introduced by a bug in Rust
// https://github.com/rust-lang/rust/issues/147648
#![allow(unused_assignments)]

use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

#[derive(Error, Diagnostic, Debug, Clone)]
pub enum XfuncError {
    #[error("An unexpected internal error occurred: {message}")]
    #[diagnostic(code("E-XXX"))]
    /// This error should not occur.
    /// Some internal invariant has been violated.
    Impossible {
        message: String,
        #[label]
        span: Option<SourceSpan>,
    },
}
