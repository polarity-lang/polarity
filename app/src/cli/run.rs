use std::path::PathBuf;

use core::eval::Eval;
use printer::PrintToString;
use source::{Database, File};
use syntax::common::*;
use syntax::ctx::Context;
use syntax::env::Env;

use crate::result::IOError;

#[derive(clap::Args)]
pub struct Args {
    #[clap(value_parser, value_name = "FILE")]
    filepath: PathBuf,
}

pub fn exec(cmd: Args) -> miette::Result<()> {
    let mut db = Database::default();
    let file = File::read(&cmd.filepath).map_err(IOError::from).map_err(miette::Report::from)?;
    let view = db.add(file).query();
    let tst = view.tst().map_err(|err| view.pretty_error(err))?;
    let prg = tst.forget();
    if let Some(exp) = &prg.exp {
        let mut env = Env::empty();
        let val = exp.eval(&prg, &mut env).unwrap();
        println!("{}", val.print_to_string());
    }
    Ok(())
}
