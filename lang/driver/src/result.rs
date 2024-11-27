use std::sync::Arc;

use miette::Diagnostic;
use thiserror::Error;
use url::Url;

#[derive(Error, Diagnostic, Debug, Clone)]
#[diagnostic(transparent)]
#[error(transparent)]
pub enum Error {
    Parser(#[from] parser::ParseError),
    Lowering(#[from] lowering::LoweringError),
    Type(#[from] Box<elaborator::result::TypeError>),
    Xfunc(#[from] transformations::result::XfuncError),
    Driver(#[from] DriverError),
}

#[derive(Error, Debug, Diagnostic, Clone)]
pub enum DriverError {
    #[error("Import cycle detected for module {0:?}: {1:?}")]
    ImportCycle(Url, Vec<Url>),
    #[error("Invalid URI: {0}")]
    InvalidUri(Url),
    #[error("File not found: {0}")]
    FileNotFound(Url),
    #[error("IO error: {0}")]
    Io(#[from] Arc<std::io::Error>),
    #[error("Tokio join error: {0}")]
    TokioJoin(#[from] Arc<tokio::task::JoinError>),
    #[error("URL error: {0}")]
    Url(#[from] url::ParseError),
    #[error("Impossible: {0}")]
    Impossible(String),
}
