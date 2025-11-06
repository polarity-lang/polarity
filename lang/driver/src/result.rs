use std::sync::Arc;

use miette::{Diagnostic, SourceSpan};
use thiserror::Error;
use url::Url;

use polarity_lang_backend::result::BackendError;

/// A Result type that can contain multiple errors from any stage of the pipeline
pub type AppResult<T = ()> = Result<T, AppErrors>;

/// A non-empty vector of [AppError]s
#[derive(Debug, Clone)]
pub struct AppErrors(Vec<AppError>);

impl AppErrors {
    pub fn from_single_error(error: AppError) -> Self {
        Self(vec![error])
    }

    pub fn from_errors(errors: Vec<AppError>) -> Self {
        if errors.is_empty() {
            let impossible = DriverError::Impossible("Empty / unknown error".to_string());
            Self::from_single_error(impossible.into())
        } else {
            Self(errors)
        }
    }

    pub fn into_errors(self) -> Vec<AppError> {
        self.0
    }
}

impl<T: Into<AppError>> From<T> for AppErrors {
    fn from(value: T) -> Self {
        AppErrors::from_single_error(value.into())
    }
}

/// The sum of all single errors that can occur in the pipeline
#[derive(Error, Diagnostic, Debug, Clone)]
#[error(transparent)]
#[diagnostic(transparent)]
pub enum AppError {
    Parser(#[from] polarity_lang_parser::ParseError),
    Lowering(#[from] Box<polarity_lang_lowering::LoweringError>),
    Type(#[from] Box<polarity_lang_elaborator::result::TypeError>),
    Xfunc(#[from] polarity_lang_transformations::result::XfuncError),
    Driver(#[from] DriverError),
    Backend(#[from] BackendError),
}

/// An error that can occur in the driver itself
#[derive(Error, Debug, Diagnostic, Clone)]
pub enum DriverError {
    #[error("Could not find module {}", import)]
    #[diagnostic(code("D-001"))]
    InvalidImport {
        #[label]
        span: SourceSpan,
        import: String,
    },
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
