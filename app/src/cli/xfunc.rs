use std::fs;
use std::io;
use std::path::PathBuf;

use source::{Database, Xfunc};

use crate::result::IOError;

#[derive(clap::Args)]
pub struct Args {
    #[clap(value_parser, value_name = "TYPE")]
    r#type: String,
    #[clap(value_parser, value_name = "FILE")]
    filepath: PathBuf,
    #[clap(short, long, value_name = "FILE")]
    output: Option<PathBuf>,
}

pub fn exec(cmd: Args) -> miette::Result<()> {
    let mut db = Database::default();
    let file =
        source::File::read(&cmd.filepath).map_err(IOError::from).map_err(miette::Report::from)?;
    let view = db.add(file).query();
    // FIXME: Proper error handling
    let Xfunc { edits, .. } = view.xfunc(&cmd.r#type).unwrap();

    let output = view.edited(edits);

    // Write to file or to stdout
    let stream: Box<dyn io::Write> = match cmd.output {
        Some(path) => Box::new(fs::File::create(path).expect("Failed to create file")),
        None => Box::new(io::stdout()),
    };

    output.write_to(stream).expect("Failed to write file");

    Ok(())
}
