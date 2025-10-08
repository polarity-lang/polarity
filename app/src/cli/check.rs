use std::path::PathBuf;

use polarity_lang_driver::Database;

#[derive(clap::Args)]
pub struct Args {
    #[clap(value_parser, value_name = "FILE")]
    filepath: PathBuf,
}

pub async fn exec(cmd: Args) -> miette::Result<()> {
    let mut db = Database::from_path(&cmd.filepath);
    let uri = db.resolve_path(&cmd.filepath)?;
    let _ = db.ast(&uri).await.map_err(|err| db.pretty_error(&uri, err))?;
    println!("{} typechecked successfully!", cmd.filepath.display());
    Ok(())
}
