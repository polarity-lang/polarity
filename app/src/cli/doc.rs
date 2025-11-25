use std::fs;
use std::path::PathBuf;

use polarity_lang_docs::get_target_path;
use polarity_lang_docs::open;
use polarity_lang_docs::write_html;

use crate::cli::locate_libs::locate_libs;

#[derive(clap::Args)]
pub struct Args {
    #[clap(value_parser, value_name = "FILE")]
    filepath: PathBuf,
    #[clap(long, num_args = 0)]
    open: bool,
    #[clap(long)]
    lib_path: Option<Vec<PathBuf>>,
}

pub async fn exec(cmd: Args) -> Result<(), Vec<miette::Report>> {
    let filepath = &cmd.filepath;
    let htmlpath =
        get_target_path(&fs::canonicalize(filepath).expect("failed to canonicalize path"));
    let lib_paths = locate_libs(cmd.lib_path);
    write_html(&lib_paths).await;
    if cmd.open {
        open(&htmlpath);
    }
    Ok(())
}
