use clap::{Parser, Subcommand};

mod check;
mod clean;
mod doc;
mod format;
mod generate_completion;
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
    let fut = async {
        match cli.command {
            Run(args) => run::exec(args).await,
            Check(args) => check::exec(args).await,
            Fmt(args) => format::exec(args).await,
            Texify(args) => texify::exec(args).await,
            Xfunc(args) => xfunc::exec(args).await,
            Lsp(args) => lsp::exec(args).await,
            Lift(args) => lift::exec(args).await,
            Doc(args) => doc::exec(args).await,
            Clean => clean::exec().await,
            GenerateCompletion(args) => generate_completion::exec(args).await,
        }
    };

    tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap().block_on(fut)
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
    /// Generate documentation for a file
    Doc(doc::Args),
    /// Clean target_pol directory
    Clean,
    /// Generate completion script for bash
    GenerateCompletion(generate_completion::Args),
}
