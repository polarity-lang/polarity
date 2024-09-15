pub mod cst;
mod grammar;
mod lexer;
mod result;

use lexer::Lexer;
use url::Url;

use grammar::cst::{ExpParser, ModuleContentsParser};
pub use result::*;

pub fn parse_exp(s: &str) -> Result<Box<cst::exp::Exp>, ParseError> {
    let lexer = Lexer::new(s);
    let parser = ExpParser::new();
    parser.parse(lexer).map_err(From::from)
}

pub fn parse_module(uri: Url, s: &str) -> Result<cst::decls::Module, ParseError> {
    let lexer = Lexer::new(s);
    let parser = ModuleContentsParser::new();
    let (use_decls, decls) = parser.parse(lexer)?;
    Ok(cst::decls::Module { uri, use_decls, decls })
}
