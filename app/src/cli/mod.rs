use clap::{Parser, Subcommand};

mod format;
mod prompt;
mod repl;
mod run;
mod terminal;

pub fn exec() {
    use Command::*;
    let cli = Cli::parse();
    core::tracer::set_enabled(cli.trace);
    match cli.command {
        Some(cmd) => match cmd {
            Run(args) => run::exec(args),
            Repl(args) => repl::exec(args),
            Fmt(args) => format::exec(args),
        },
        None => repl::exec(repl::Args::default()),
    }
}

#[derive(Parser)]
#[clap(author, about, version=crate::VERSION, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Option<Command>,
    /// Enable internal debug output
    #[clap(long, takes_value = false)]
    trace: bool,
}

#[derive(Subcommand)]
enum Command {
    /// Run a source code file
    Run(run::Args),
    /// Start an interactive console
    Repl(repl::Args),
    /// Format/pretty-print a code file
    Fmt(format::Args),
}
