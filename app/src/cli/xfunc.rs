use std::fs;
use std::io;
use std::path::PathBuf;

use polarity_lang_driver::{Database, Xfunc};

use crate::cli::locate_libs::locate_libs;

#[derive(clap::Args)]
pub struct Args {
    #[clap(value_parser, value_name = "TYPE")]
    r#type: String,
    #[clap(value_parser, value_name = "FILE")]
    filepath: PathBuf,
    #[clap(short, long, value_name = "FILE")]
    output: Option<PathBuf>,
    #[clap(long)]
    lib_path: Option<Vec<PathBuf>>,
}

pub async fn exec(cmd: Args) -> Result<(), Vec<miette::Report>> {
    let lib_paths = locate_libs(cmd.lib_path);
    let mut db = Database::from_path(&cmd.filepath, &lib_paths);
    let uri = db.resolve_path(&cmd.filepath).map_err(|e| vec![e.into()])?;
    let Xfunc { edits, .. } =
        db.xfunc(&uri, &cmd.r#type).await.map_err(|errs| db.pretty_errors(&uri, errs))?;

    let output = db.edited(&uri, edits);

    // Write to file or to stdout
    let stream: Box<dyn io::Write> = match cmd.output {
        Some(path) => Box::new(fs::File::create(path).expect("Failed to create file")),
        None => Box::new(io::stdout()),
    };

    output.write_to(stream).expect("Failed to write file");

    Ok(())
}
