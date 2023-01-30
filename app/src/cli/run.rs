use std::path::PathBuf;

use printer::{ColorChoice, PrintExt, StandardStream};
use source::{Database, File};
use syntax::val;

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
    let val = view.run().map_err(|err| view.pretty_error(err))?;
    if let Some(val) = val {
        print_val(&val)
    }
    Ok(())
}

fn print_val(val: &val::Val) {
    let mut stream = StandardStream::stdout(ColorChoice::Auto);
    val.print_colored(&Default::default(), &mut stream).expect("Failed to print to stdout");
    println!();
}
