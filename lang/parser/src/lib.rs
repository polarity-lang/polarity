pub mod cst;
mod grammar;
mod lexer;
mod result;

use std::rc::Rc;

use lexer::Lexer;
use url::Url;

use grammar::cst::{DeclsParser, ExpParser};
pub use result::*;

pub fn parse_exp(s: &str) -> Result<Rc<cst::exp::Exp>, ParseError> {
    let lexer = Lexer::new(s);
    let parser = ExpParser::new();
    parser.parse(lexer).map_err(From::from)
}

pub fn parse_module(uri: Url, s: &str) -> Result<cst::decls::Module, ParseError> {
    let lexer = Lexer::new(s);
    let parser = DeclsParser::new();
    let items = parser.parse(lexer)?;
    Ok(cst::decls::Module { uri, items })
}
