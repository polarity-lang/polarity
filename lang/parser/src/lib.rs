pub mod cst;
mod grammar;
mod result;

use std::{path::Path, rc::Rc};

use grammar::cst::{DeclsParser, ExpParser};
pub use result::*;
use url::Url;

pub fn parse_exp(s: &str) -> Result<Rc<cst::exp::Exp>, ParseError> {
    ExpParser::new().parse(s).map_err(From::from)
}

pub fn parse_module(fp: &Path, s: &str) -> Result<cst::decls::Module, ParseError> {
    let uri = Url::from_file_path(fp).map_err(|_| ParseError::User {
        error: format!("Cannot convert filepath {:?} to url", fp),
    })?;
    let decls = DeclsParser::new().parse(s)?;
    Ok(cst::decls::Module { uri, items: decls })
}
