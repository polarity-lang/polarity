use miette::Diagnostic;
use thiserror::Error;

use syntax::common::*;

#[derive(Error, Diagnostic, Debug)]
pub enum LoweringError {
    #[error("Undefined identifier {name}")]
    UndefinedIdent { name: Ident },
    #[error("Duplicate definition of {name}")]
    AlreadyDefined { name: Ident },
    #[error("{name} must be used as destructor")]
    MustUseAsDtor { name: Ident },
    #[error("Arguments to type constructor {typ} must be provided for {xtor}")]
    MustProvideArgs { xtor: Ident, typ: Ident },
}
