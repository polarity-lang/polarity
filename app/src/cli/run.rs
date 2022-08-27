use std::path::PathBuf;

use crate::result::HandleErrorExt;

use super::terminal;

#[derive(clap::Args)]
pub struct Args {
    #[clap(value_parser, value_name = "FILE")]
    filepath: PathBuf,
}

pub fn exec(cmd: Args) {
    crate::rt::run_filepath(&cmd.filepath).handle(terminal::print_prg)
}
