use std::fs;
use std::path::Path;

use syntax::ast;

use crate::result::Error;

pub fn run_filepath(filepath: &Path) -> Result<ast::Prg, Error> {
    let prg = load_filepath(filepath)?;
    lowering::lower(prg).map_err(Error::Lowering)
}

pub fn run_string(text: &str) -> Result<ast::Prg, Error> {
    let prg = load_string(text)?;
    lowering::lower(prg).map_err(Error::Lowering)
}

fn load_filepath(filepath: &Path) -> Result<syntax::cst::Prg, Error> {
    let text = fs::read_to_string(filepath).map_err(Error::IO)?;
    let prg = load_string(&text)?;
    Ok(prg)
}

pub fn load_string(text: &str) -> Result<syntax::cst::Prg, Error> {
    parser::cst::parse_program(text).map_err(Error::Parser)
}
