use std::path::PathBuf;

use polarity_lang_driver::Database;

#[derive(clap::Args)]
pub struct Args {
    #[clap(value_parser, value_name = "FILE")]
    filepath: PathBuf,
}

pub async fn exec(cmd: Args) -> Result<(), Vec<miette::Report>> {
    let mut db = Database::from_path(&cmd.filepath);
    let uri = db.resolve_path(&cmd.filepath).map_err(|e| vec![e.into()])?;
    let _ = db.ast(&uri).await.map_err(|errs| db.pretty_errors(&uri, errs))?;
    println!("{} typechecked successfully!", cmd.filepath.display());
    Ok(())
}
