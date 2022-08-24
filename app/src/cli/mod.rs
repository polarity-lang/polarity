use clap::{Parser, Subcommand};

mod prompt;
mod repl;
mod run;

pub fn exec() {
    use Command::*;
    let cli = Cli::parse();
    match cli.command {
        Some(cmd) => match cmd {
            Run(args) => run::exec(args),
            Repl(args) => repl::exec(args),
        },
        None => repl::exec(repl::Args::default()),
    }
}

#[derive(Parser)]
#[clap(author, about, version=crate::VERSION, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Run a source code file
    Run(run::Args),
    /// Start an interactive console
    Repl(repl::Args),
}
