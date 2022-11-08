use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

use syntax::common::*;

#[derive(Error, Diagnostic, Debug)]
#[diagnostic()]
pub enum LoweringError {
    #[error("Undefined identifier {name}")]
    UndefinedIdent {
        name: Ident,
        #[label]
        span: SourceSpan,
    },
    #[error("Duplicate definition of {name}")]
    AlreadyDefined {
        name: Ident,
        #[label]
        span: Option<SourceSpan>,
    },
    #[error("{name} must be used as destructor")]
    MustUseAsDtor {
        name: Ident,
        #[label]
        span: SourceSpan,
    },
    #[error("Arguments to type constructor {typ} must be provided for {xtor}")]
    MustProvideArgs {
        xtor: Ident,
        typ: Ident,
        #[label]
        span: SourceSpan,
    },
    #[error("Local (co)match must be named")]
    UnnamedMatch {
        #[label]
        span: SourceSpan,
    },
}
