use clap::{Parser, Subcommand};

use crate::global_settings::GlobalSettings;

mod check;
mod clean;
mod compile;
mod doc;
mod format;
mod gen_completions;
mod lex;
mod lift;
mod lsp;
mod run;
mod texify;
mod xfunc;

pub fn exec(settings: &mut GlobalSettings) -> Result<(), Vec<miette::Report>> {
    let cli = Cli::parse();
    // Initialize the logger based on the flags
    let mut builder = env_logger::Builder::from_default_env();
    builder.format_timestamp(None).format_level(false).format_target(false);

    if cli.trace {
        settings.log_level = log::LevelFilter::Trace;
    } else if cli.debug {
        settings.log_level = log::LevelFilter::Debug;
    }
    builder.filter_level(settings.log_level);

    builder.init();

    match cli.colorize {
        Some(clap::ColorChoice::Auto) => {
            settings.colorize = polarity_lang_printer::ColorChoice::Auto
        }
        Some(clap::ColorChoice::Always) => {
            settings.colorize = polarity_lang_printer::ColorChoice::Always
        }
        Some(clap::ColorChoice::Never) => {
            settings.colorize = polarity_lang_printer::ColorChoice::Never
        }
        None => (),
    }

    use Command::*;
    let fut = async {
        match cli.command {
            Run(args) => run::exec(args, settings).await,
            Check(args) => check::exec(args).await,
            Fmt(args) => format::exec(args, settings).await,
            Texify(args) => texify::exec(args).await,
            Xfunc(args) => xfunc::exec(args).await,
            Lex(args) => lex::exec(args).await,
            Lsp(args) => lsp::exec(args).await,
            Lift(args) => lift::exec(args).await,
            Doc(args) => doc::exec(args).await,
            Clean => clean::exec().await,
            GenerateCompletion(args) => gen_completions::exec(args).await,
            Compile(args) => compile::exec(args).await,
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
    #[clap(long)]
    colorize: Option<clap::ColorChoice>,
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
    /// Lex a file and print the resulting token stream for debugging
    #[clap(hide(true))]
    Lex(lex::Args),
    /// Lift local (co)matches of a type to the top-level
    Lift(lift::Args),
    /// Generate documentation for a file
    Doc(doc::Args),
    /// Clean target_pol directory
    Clean,
    /// Generate completion scripts for various shells
    GenerateCompletion(gen_completions::Args),
    /// Compile an executable
    Compile(compile::Args),
}
