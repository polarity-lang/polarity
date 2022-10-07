use std::fmt;

#[derive(Debug)]
pub enum Error {
    Parser(parser::ParseError<usize, parser::common::OwnedToken, &'static str>),
    Lowering(lowering::LoweringError),
    Type(core::TypeError),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Parser(err) => write!(f, "Parse error: {}", err),
            Error::Lowering(err) => write!(f, "Lowering error: {}", err),
            Error::Type(err) => write!(f, "Type error: {}", err),
        }
    }
}
