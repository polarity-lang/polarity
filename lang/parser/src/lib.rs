pub mod cst;
mod grammar;
mod result;

use std::rc::Rc;

use grammar::cst::{ExpParser, PrgParser};
pub use result::*;

pub fn parse_exp(s: &str) -> Result<Rc<cst::exp::Exp>, ParseError> {
    ExpParser::new().parse(s).map_err(From::from)
}

pub fn parse_module(s: &str) -> Result<cst::decls::Module, ParseError> {
    PrgParser::new().parse(s).map_err(From::from)
}
