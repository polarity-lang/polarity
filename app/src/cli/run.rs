use std::io;
use std::path::PathBuf;
use std::process::{Command, ExitStatus};

use miette::Diagnostic;
use thiserror::Error;

use polarity_lang_driver::Database;
use polarity_lang_printer::{ColorChoice, Print, StandardStream};

use crate::global_settings::GlobalSettings;
use crate::utils::codegen;

#[derive(clap::Args)]
pub struct Args {
    #[clap(value_parser, value_name = "FILE")]
    filepath: PathBuf,

    /// Only output the normal form of main and don't compile.
    #[clap(long, short)]
    normalize: bool,

    /// Arguments to pass to the executed program.
    #[clap(trailing_var_arg = true)]
    args: Vec<String>,
}

pub async fn exec(cmd: Args, settings: &GlobalSettings) -> Result<(), Vec<miette::Report>> {
    let mut db = Database::from_path(&cmd.filepath);
    let uri = db.resolve_path(&cmd.filepath).map_err(|e| vec![e.into()])?;

    if cmd.normalize {
        let nf = db.normalize_main(&uri).await.map_err(|errs| db.pretty_errors(&uri, errs))?;
        match nf {
            Some(nf) => print_nf(&nf, settings.colorize),
            None => return Err(vec![miette::Report::from(MainNotFound {})]),
        }
    } else {
        codegen::generate_ir(&mut db, &cmd.filepath).await?;
        let js_file = codegen::generate_js(&mut db, &cmd.filepath).await?;

        run_node(&js_file, &cmd.args).map_err(|err| vec![NodeFailed { err }.into()])?;
    }

    Ok(())
}

fn print_nf(nf: &polarity_lang_ast::Exp, colorize: ColorChoice) {
    let mut stream = StandardStream::stdout(colorize);
    nf.print_colored(&Default::default(), &mut stream).expect("Failed to print to stdout");
    println!();
}

fn run_node(path: &PathBuf, args: &[String]) -> io::Result<ExitStatus> {
    Command::new("node").arg(path).args(args).status()
}

#[derive(Error, Diagnostic, Debug)]
#[error("Main expression was not found")]
#[diagnostic(help("Main expressions must be called \"main\" and not take any arguments."))]
pub struct MainNotFound {}

#[derive(Error, Diagnostic, Debug)]
#[error("Failed to run `node` on JS output: {err}")]
pub struct NodeFailed {
    err: io::Error,
}
