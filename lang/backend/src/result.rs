use miette::Diagnostic;
use thiserror::Error;

#[derive(Error, Debug, Diagnostic, Clone)]
pub enum BackendError {
    #[error("Impossible: {0}")]
    Impossible(String),
    #[error("Code generation error: {0}")]
    CodegenError(String),
}

pub type BackendResult<T = ()> = Result<T, BackendError>;
