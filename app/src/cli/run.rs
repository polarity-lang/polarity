use std::path::PathBuf;

use source::{Database, File};

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
    view.tst().map_err(|err| view.pretty_error(err))?;
    Ok(())
}
