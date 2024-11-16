use clap::{Parser, Subcommand};

mod check;
mod format;
mod lift;
mod lsp;
mod run;
mod texify;
mod xfunc;

pub fn exec() -> miette::Result<()> {
    let cli = Cli::parse();
    // Initialize the logger based on the flags
    let mut builder = env_logger::Builder::from_default_env();
    builder.format_timestamp(None).format_level(false).format_target(false);

    if cli.trace {
        builder.filter_level(log::LevelFilter::Trace);
    } else if cli.debug {
        builder.filter_level(log::LevelFilter::Debug);
    } else {
        builder.filter_level(log::LevelFilter::Info);
    }

    builder.init();

    use Command::*;
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
    /// Enable trace logging
    #[clap(long)]
    trace: bool,
    /// Enable debug logging
    #[clap(long)]
    debug: bool,
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
    #[clap(hide(true))]
    Lsp(lsp::Args),
    /// Lift local (co)matches of a type to the top-level
    Lift(lift::Args),
}
