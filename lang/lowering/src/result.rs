use std::error::Error;
use std::fmt;

use syntax::common::*;

#[derive(Debug)]
pub enum LoweringError {
    UndefinedIdent(Ident),
    AlreadyDefined(Ident),
    MustUseAsDtor(Ident),
    MustProvideArgs { xtor: Ident, typ: Ident },
}

impl fmt::Display for LoweringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UndefinedIdent(ident) => write!(f, "Undefined identifier {ident}"),
            Self::AlreadyDefined(ident) => write!(f, "Duplicate definition of {ident}"),
            // TODO: Improve error message
            Self::MustUseAsDtor(ident) => write!(f, "{ident} must be used as destructor"),
            Self::MustProvideArgs { xtor, typ } => {
                write!(f, "Arguments to type constructor {typ} must be provided for {xtor}")
            }
        }
    }
}

impl Error for LoweringError {}
