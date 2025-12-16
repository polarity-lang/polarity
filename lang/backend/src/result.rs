use miette::Diagnostic;
use thiserror::Error;

#[derive(Error, Debug, Diagnostic, Clone)]
pub enum BackendError {
    #[error("Code generation error: {0}")]
    CodegenError(String),
    #[error("The feature '{feature}' is not implemented for the '{backend}' backend")]
    Unimplemented { feature: String, backend: String },
    #[error("Impossible: {0}")]
    Impossible(String),
}

pub type BackendResult<T = ()> = Result<T, BackendError>;
