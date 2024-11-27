use std::fs;
use std::io;
use std::path::PathBuf;

use driver::{Database, Xfunc};

#[derive(clap::Args)]
pub struct Args {
    #[clap(value_parser, value_name = "TYPE")]
    r#type: String,
    #[clap(value_parser, value_name = "FILE")]
    filepath: PathBuf,
    #[clap(short, long, value_name = "FILE")]
    output: Option<PathBuf>,
}

pub async fn exec(cmd: Args) -> miette::Result<()> {
    let mut db = Database::from_path(&cmd.filepath);
    let uri = db.resolve_path(&cmd.filepath)?;
    let Xfunc { edits, .. } = db.xfunc(&uri, &cmd.r#type).await.map_err(miette::Report::msg)?;

    let output = db.edited(&uri, edits);

    // Write to file or to stdout
    let stream: Box<dyn io::Write> = match cmd.output {
        Some(path) => Box::new(fs::File::create(path).expect("Failed to create file")),
        None => Box::new(io::stdout()),
    };

    output.write_to(stream).expect("Failed to write file");

    Ok(())
}
