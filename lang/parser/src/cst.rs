use std::rc::Rc;

use super::grammar::cst::{ExpParser, PrgParser};
use super::result::ParseError;

pub fn parse_exp(s: &str) -> Result<Rc<syntax::cst::Exp>, ParseError> {
    ExpParser::new().parse(s).map_err(From::from)
}

pub fn parse_program(s: &str) -> Result<syntax::cst::Prg, ParseError> {
    PrgParser::new().parse(s).map_err(From::from)
}
