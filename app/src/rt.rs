use std::fs;
use std::path::Path;

use syntax::cst;
use syntax::tst;
use syntax::ust;

use crate::result::Error;

pub fn run_filepath(filepath: &Path) -> Result<tst::Prg, Error> {
    let prg = load_filepath(filepath)?;
    run_program(prg)
}

pub fn run_string(text: &str) -> Result<tst::Prg, Error> {
    let prg = load_string(text)?;
    run_program(prg)
}

pub fn lower_filepath(filepath: &Path) -> Result<ust::Prg, Error> {
    let prg = load_filepath(filepath)?;
    lowering::lower(&prg).map_err(Error::Lowering)
}

fn run_program(prg: cst::Prg) -> Result<tst::Prg, Error> {
    let ast = lowering::lower(&prg).map_err(Error::Lowering)?;
    core::check(&ast).map_err(Error::Type)
}

fn load_filepath(filepath: &Path) -> Result<syntax::cst::Prg, Error> {
    let text = fs::read_to_string(filepath).map_err(Error::IO)?;
    let prg = load_string(&text)?;
    Ok(prg)
}

pub fn load_string(text: &str) -> Result<syntax::cst::Prg, Error> {
    parser::cst::parse_program(text).map_err(Error::Parser)
}
