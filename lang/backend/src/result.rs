use miette::Diagnostic;
use thiserror::Error;

use crate::ir::rename::RenameError;

#[derive(Error, Debug, Diagnostic, Clone)]
pub enum BackendError {
    #[error("Code generation error: {0}")]
    CodegenError(String),
    #[error("An internal error occured while renaming the IR.")]
    RenameError(#[from] RenameError),
    #[error("Impossible: {0}")]
    Impossible(String),
}

pub type BackendResult<T = ()> = Result<T, BackendError>;
