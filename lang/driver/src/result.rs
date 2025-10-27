use std::sync::Arc;

use miette::Diagnostic;
use thiserror::Error;
use url::Url;

use polarity_lang_backend::result::BackendError;

#[derive(Error, Diagnostic, Debug, Clone)]
#[error(transparent)]
pub enum Error {
    #[error("Failed parsing")]
    Parser(#[related] Vec<polarity_lang_parser::ParseError>),

    #[diagnostic(transparent)]
    Lowering(#[from] Box<polarity_lang_lowering::LoweringError>),

    #[diagnostic(transparent)]
    Type(#[from] Box<polarity_lang_elaborator::result::TypeError>),

    #[diagnostic(transparent)]
    Xfunc(#[from] polarity_lang_transformations::result::XfuncError),

    #[diagnostic(transparent)]
    Driver(#[from] DriverError),

    #[diagnostic(transparent)]
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
