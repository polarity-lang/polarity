pub mod cst;
mod grammar;
mod lexer;
mod result;

use std::rc::Rc;

use url::Url;

use grammar::cst::{DeclsParser, ExpParser};
pub use result::*;

pub fn parse_exp(s: &str) -> Result<Rc<cst::exp::Exp>, ParseError> {
    ExpParser::new().parse(s).map_err(From::from)
}

pub fn parse_module(uri: Url, s: &str) -> Result<cst::decls::Module, ParseError> {
    let items = DeclsParser::new().parse(s)?;
    Ok(cst::decls::Module { uri, items })
}
