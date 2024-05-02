use std::path::PathBuf;

use miette::Diagnostic;
use thiserror::Error;

use printer::{ColorChoice, PrintExt, StandardStream};
use query::{Database, File};
use syntax::generic;

use crate::result::IOError;

#[derive(clap::Args)]
pub struct Args {
    #[clap(value_parser, value_name = "FILE")]
    filepath: PathBuf,
}

pub fn exec(cmd: Args) -> miette::Result<()> {
    let mut db = Database::default();
    let file = File::read(&cmd.filepath).map_err(IOError::from).map_err(miette::Report::from)?;
    let view = db.add(file).query();

    let nf = view.run().map_err(|err| view.pretty_error(err))?;
    match nf {
        Some(nf) => print_nf(&nf),
        None => return Err(miette::Report::from(MainNotFound {})),
    }
    Ok(())
}

fn print_nf(nf: &generic::Exp) {
    let mut stream = StandardStream::stdout(ColorChoice::Auto);
    nf.print_colored(&Default::default(), &mut stream).expect("Failed to print to stdout");
    println!();
}

#[derive(Error, Diagnostic, Debug)]
#[error("Main expression was not found")]
#[diagnostic(help("Main expressions must be called \"main\" and not take any arguments."))]
pub struct MainNotFound {}
