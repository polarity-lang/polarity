use clap::{Parser, Subcommand};

mod run;

pub fn exec() {
    use Command::*;
    let cli = Cli::parse();
    core::tracer::set_enabled(cli.trace);
    match cli.command {
        Run(args) => run::exec(args),
    }
}

#[derive(Parser)]
#[clap(author, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
    /// Enable internal debug output
    #[clap(long, takes_value = false)]
    trace: bool,
}

#[derive(Subcommand)]
enum Command {
    /// Run all test suites
    Run(run::Args),
}
