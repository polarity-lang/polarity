use std::path::PathBuf;

use polarity_lang_driver::Database;

use crate::utils::codegen;

#[derive(clap::Args)]
pub struct Args {
    #[clap(value_parser, value_name = "FILE")]
    filepath: PathBuf,
}

pub async fn exec(cmd: Args) -> Result<(), Vec<miette::Report>> {
    let mut db = Database::from_path(&cmd.filepath);

    codegen::generate_ir(&mut db, &cmd.filepath).await?;
    codegen::generate_js(&mut db, &cmd.filepath).await?;

    Ok(())
}
