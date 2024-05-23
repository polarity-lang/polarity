use clap::{Parser, Subcommand};

mod check;
mod format;
mod ignore_colors;
mod lift;
mod lsp;
mod run;
mod texify;
mod xfunc;

pub fn exec() -> miette::Result<()> {
    use Command::*;
    let cli = Cli::parse();
    match cli.command {
        Run(args) => run::exec(args),
        Check(args) => check::exec(args),
        Fmt(args) => format::exec(args),
        Texify(args) => texify::exec(args),
        Xfunc(args) => xfunc::exec(args),
        Lsp(args) => lsp::exec(args),
        Lift(args) => lift::exec(args),
    }
}

#[derive(Parser)]
#[clap(version, author, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run the main expression of a file
    Run(run::Args),
    /// Typecheck a file
    Check(check::Args),
    /// Format a code file
    Fmt(format::Args),
    /// Render a code file as a latex document
    Texify(texify::Args),
    /// De-/Refunctionalize a type in a code file
    Xfunc(xfunc::Args),
    /// Start an LSP server
    Lsp(lsp::Args),
    /// Lift local (co)matches of a type to the top-level
    Lift(lift::Args),
}
