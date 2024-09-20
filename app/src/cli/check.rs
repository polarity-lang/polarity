use std::path::PathBuf;

use query::Database;

#[derive(clap::Args)]
pub struct Args {
    #[clap(value_parser, value_name = "FILE")]
    filepath: PathBuf,
}

pub fn exec(cmd: Args) -> miette::Result<()> {
    let mut db = Database::from_path(&cmd.filepath);
    let mut view = db.open_path(&cmd.filepath)?;
    let _ = view.load_module().map_err(|err| view.pretty_error(err))?;
    println!("{} typechecked successfully!", cmd.filepath.display());
    Ok(())
}
