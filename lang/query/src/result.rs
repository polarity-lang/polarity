use miette::Diagnostic;
use thiserror::Error;

#[derive(Error, Diagnostic, Debug)]
#[diagnostic(transparent)]
#[error(transparent)]
pub enum Error {
    Parser(#[from] parser::ParseError),
    Lowering(#[from] lowering::LoweringError),
    Type(#[from] elaborator::result::TypeError),
    Xfunc(#[from] xfunc::result::XfuncError),
}
