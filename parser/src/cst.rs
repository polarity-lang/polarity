use std::rc::Rc;

use lalrpop_util::{lexer::Token, ParseError};

use super::common::OwnedToken;
use super::grammar::cst::{ExpParser, PrgParser};

pub fn parse_exp(
    s: &str,
) -> Result<Rc<syntax::cst::Exp>, ParseError<usize, Token<'_>, &'static str>> {
    ExpParser::new().parse(s)
}

pub fn parse_program(
    s: &str,
) -> Result<syntax::cst::Prg, ParseError<usize, OwnedToken, &'static str>> {
    PrgParser::new().parse(s).map_err(|err| err.map_token(OwnedToken::from))
}
