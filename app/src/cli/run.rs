use std::path::PathBuf;

use printer::{ColorChoice, PrintExt, StandardStream};
use source::{Database, File};
use syntax::nf;

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
    println!("{} typechecked successfully!", cmd.filepath.display());
    if let Some(nf) = nf {
        print_nf(&nf)
    }
    Ok(())
}

fn print_nf(nf: &nf::Nf) {
    let mut stream = StandardStream::stdout(ColorChoice::Auto);
    nf.print_colored(&Default::default(), &mut stream).expect("Failed to print to stdout");
    println!();
}
