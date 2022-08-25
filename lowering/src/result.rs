use std::error::Error;
use std::fmt;

use syntax::common::*;

#[derive(Debug)]
pub enum LoweringError {
    UndefinedIdent(Ident),
    AlreadyDefined(Ident),
}

impl fmt::Display for LoweringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UndefinedIdent(ident) => write!(f, "Undefined identifier {}", ident),
            Self::AlreadyDefined(ident) => write!(f, "Duplicate definition of {}", ident),
        }
    }
}

impl Error for LoweringError {}
