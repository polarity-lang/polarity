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
    Type(#[from] elaborator::result::TypeError),
    Xfunc(#[from] xfunc::result::XfuncError),
    Driver(#[from] DriverError),
}

#[derive(Error, Debug, Diagnostic, Clone)]
pub enum DriverError {
    #[error("Invalid URI: {0}")]
    InvalidUri(Url),
    #[error("IO error: {0}")]
    Io(#[from] Arc<std::io::Error>),
    #[error("URL error: {0}")]
    Url(#[from] url::ParseError),
}
