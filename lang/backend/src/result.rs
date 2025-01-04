use miette::Diagnostic;
use thiserror::Error;

#[derive(Error, Debug, Diagnostic, Clone)]
pub enum BackendError {
    #[error("Impossible: {0}")]
    Impossible(String),
}
