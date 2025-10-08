use std::sync::Arc;

use miette::Diagnostic;
use thiserror::Error;
use url::Url;

use polarity_lang_backend::result::BackendError;

#[derive(Error, Diagnostic, Debug, Clone)]
#[diagnostic(transparent)]
#[error(transparent)]
pub enum Error {
    Parser(#[from] polarity_lang_parser::ParseError),
    Lowering(#[from] Box<polarity_lang_lowering::LoweringError>),
    Type(#[from] Box<polarity_lang_elaborator::result::TypeError>),
    Xfunc(#[from] polarity_lang_transformations::result::XfuncError),
    Driver(#[from] DriverError),
    Backend(#[from] BackendError),
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
    #[error("URL error: {0}")]
    Url(#[from] url::ParseError),
    #[error("Impossible: {0}")]
    Impossible(String),
    #[error("The file is present, but does not contain the specified byte index.")]
    IndexTooLarge { given: usize, max: usize },
    #[error("The file is present, but does not contain the specified line index.")]
    LineTooLarge { given: usize, max: usize },
    #[error(
        "The given index is contained in the file, but is not a boundary of a UTF-8 code point."
    )]
    InvalidCharBoundary { given: usize },
}
