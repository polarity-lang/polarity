use std::path::PathBuf;

use miette::Diagnostic;
use thiserror::Error;

use polarity_lang_driver::Database;
use polarity_lang_printer::{ColorChoice, Print, StandardStream};

#[derive(clap::Args)]
pub struct Args {
    #[clap(value_parser, value_name = "FILE")]
    filepath: PathBuf,
}

pub async fn exec(cmd: Args) -> Result<(), Vec<miette::Report>> {
    let mut db = Database::from_path(&cmd.filepath);
    // TODO
    let uri = db.resolve_path(&cmd.filepath).unwrap();

    let nf = db.run(&uri).await.map_err(|errs| db.pretty_errors(&uri, errs))?;

    match nf {
        Some(nf) => print_nf(&nf),
        None => return Err(vec![miette::Report::from(MainNotFound {})]),
    }
    Ok(())
}

fn print_nf(nf: &polarity_lang_ast::Exp) {
    let mut stream = StandardStream::stdout(ColorChoice::Auto);
    nf.print_colored(&Default::default(), &mut stream).expect("Failed to print to stdout");
    println!();
}

#[derive(Error, Diagnostic, Debug)]
#[error("Main expression was not found")]
#[diagnostic(help("Main expressions must be called \"main\" and not take any arguments."))]
pub struct MainNotFound {}
