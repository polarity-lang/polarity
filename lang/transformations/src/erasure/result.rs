use miette::Diagnostic;
use thiserror::Error;

#[derive(Error, Debug, Diagnostic, Clone)]
pub enum ErasureError {
    #[error("Impossible: {0}")]
    Impossible(String),
}
