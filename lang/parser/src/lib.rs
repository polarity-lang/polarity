pub mod cst;
mod grammar;
pub mod lexer;
mod result;

use lexer::Lexer;
use url::Url;

use grammar::cst::{ExpParser, ModuleContentsParser};
pub use result::*;

pub fn parse_exp(s: &str) -> Result<Box<cst::exp::Exp>, Vec<ParseError>> {
    let lexer = Lexer::new(s);
    let parser = ExpParser::new();
    let mut errors = Vec::new();

    // `parser.parse` only *returns* an error if error recovery failed (e.g. a lexing error).
    // Recovered errors are passed into the `errors` Vec.
    // For the moment, we have to catch all errors at parsing stage here.
    let result = parser.parse(&mut errors, lexer).map_err(|e| vec![e.into()]);
    if !errors.is_empty() {
        return Err(errors.into_iter().map(|e| e.error.into()).collect());
    }

    result
}

pub fn parse_module(uri: Url, s: &str) -> Result<cst::decls::Module, Vec<ParseError>> {
    let lexer = Lexer::new(s);
    let parser = ModuleContentsParser::new();
    let mut errors = Vec::new();

    // `parser.parse` only *returns* an error if error recovery failed (e.g. a lexing error).
    // Recovered errors are passed into the `errors` Vec.
    // For the moment, we have to catch all errors at parsing stage here.
    let (use_decls, decls) = parser.parse(&mut errors, lexer).map_err(|e| vec![e.into()])?;
    if !errors.is_empty() {
        return Err(errors.into_iter().map(|e| e.error.into()).collect());
    }

    Ok(cst::decls::Module { uri, use_decls, decls })
}
