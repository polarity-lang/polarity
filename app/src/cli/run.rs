use std::path::PathBuf;

use miette::Diagnostic;
use thiserror::Error;

use printer::{ColorChoice, Print, StandardStream};
use query::Database;

#[derive(clap::Args)]
pub struct Args {
    #[clap(value_parser, value_name = "FILE")]
    filepath: PathBuf,
}

pub fn exec(cmd: Args) -> miette::Result<()> {
    let mut db = Database::from_path(&cmd.filepath);
    let mut view = db.open_path(&cmd.filepath)?;
    let nf = view.run().map_err(|err| view.pretty_error(err))?;

    match nf {
        Some(nf) => print_nf(&nf),
        None => return Err(miette::Report::from(MainNotFound {})),
    }
    Ok(())
}

fn print_nf(nf: &ast::Exp) {
    let mut stream = StandardStream::stdout(ColorChoice::Auto);
    nf.print_colored(&Default::default(), &mut stream).expect("Failed to print to stdout");
    println!();
}

#[derive(Error, Diagnostic, Debug)]
#[error("Main expression was not found")]
#[diagnostic(help("Main expressions must be called \"main\" and not take any arguments."))]
pub struct MainNotFound {}
