use clap::{Parser, Subcommand};

mod format;
mod ignore_colors;
mod run;
mod xfunc;

pub fn exec() -> miette::Result<()> {
    use Command::*;
    let cli = Cli::parse();
    core::tracer::set_enabled(cli.trace);
    match cli.command {
        Run(args) => run::exec(args),
        Fmt(args) => format::exec(args),
        Xfunc(args) => xfunc::exec(args),
    }
}

#[derive(Parser)]
#[clap(author, about, version=crate::VERSION, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
    /// Enable internal debug output
    #[clap(long, num_args = 0)]
    trace: bool,
}

#[derive(Subcommand)]
enum Command {
    /// Run a source code file
    Run(run::Args),
    /// Format/pretty-print a code file
    Fmt(format::Args),
    /// De-/Refunctionalize a type in a code file
    Xfunc(xfunc::Args),
}
