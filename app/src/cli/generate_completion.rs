use std::{io, path::PathBuf};

use clap::CommandFactory;
use clap_complete::{generate, shells::Bash};

use super::Cli;

#[derive(clap::Args)]
pub struct Args {
    #[clap(value_parser, value_name = "FILE")]
    filepath: PathBuf,
}

pub async fn exec(_cmd: Args) -> miette::Result<()> {
    generate(Bash, &mut Cli::command(), "pol", &mut io::stdout());
    Ok(())
}
