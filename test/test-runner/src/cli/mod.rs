use clap::{Parser, Subcommand};

mod run;

pub fn exec() {
    use Command::*;
    let cli = Cli::parse();
    match cli.command {
        Run(args) => run::exec(args),
    }
}

#[derive(Parser)]
#[clap(author, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run all test suites
    Run(run::Args),
}
