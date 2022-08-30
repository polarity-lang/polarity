use std::error::Error;
use std::fmt;

use syntax::common::*;

#[derive(Debug)]
pub enum LoweringError {
    UndefinedIdent(Ident),
    AlreadyDefined(Ident),
    MustUseAsDtor(Ident),
}

impl fmt::Display for LoweringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UndefinedIdent(ident) => write!(f, "Undefined identifier {}", ident),
            Self::AlreadyDefined(ident) => write!(f, "Duplicate definition of {}", ident),
            // TODO: Improve error message
            Self::MustUseAsDtor(ident) => write!(f, "{} must be used as destructor", ident),
        }
    }
}

impl Error for LoweringError {}
