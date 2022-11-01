use std::io;

use miette::Diagnostic;
use thiserror::Error;

#[derive(Error, Diagnostic, Debug)]
#[error("IO Error")]
pub struct IOError {
    #[from]
    inner: io::Error,
}
