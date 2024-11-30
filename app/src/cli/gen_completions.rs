use std::{io, path::PathBuf};

use clap::CommandFactory;
use clap_complete::{
    generate,
    shells::{Bash, Elvish, Fish, PowerShell, Zsh},
};

use super::Cli;

#[allow(clippy::enum_variant_names)]
#[derive(clap::ValueEnum, Clone)]
pub enum Shell {
    Bash,
    Elvish,
    Fish,
    PowerShell,
    Zsh,
}

#[derive(clap::Args)]
pub struct Args {
    /// Target shell
    shell: Shell,
    /// Where the completion script should be saved.
    #[clap(value_parser, value_name = "PATH")]
    filepath: PathBuf,
}

pub async fn exec(cmd: Args) -> miette::Result<()> {
    match cmd.shell {
        Shell::Bash => generate(Bash, &mut Cli::command(), "pol", &mut io::stdout()),
        Shell::Elvish => generate(Elvish, &mut Cli::command(), "pol", &mut io::stdout()),
        Shell::Fish => generate(Fish, &mut Cli::command(), "pol", &mut io::stdout()),
        Shell::PowerShell => generate(PowerShell, &mut Cli::command(), "pol", &mut io::stdout()),
        Shell::Zsh => generate(Zsh, &mut Cli::command(), "pol", &mut io::stdout()),
    }
    Ok(())
}
