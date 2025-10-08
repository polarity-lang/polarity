use std::fs;
use std::path::PathBuf;

use polarity_lang_docs::get_target_path;
use polarity_lang_docs::open;
use polarity_lang_docs::write_html;

#[derive(clap::Args)]
pub struct Args {
    #[clap(value_parser, value_name = "FILE")]
    filepath: PathBuf,
    #[clap(long, num_args = 0)]
    open: bool,
}

pub async fn exec(cmd: Args) -> miette::Result<()> {
    let filepath = &cmd.filepath;
    let htmlpath =
        get_target_path(&fs::canonicalize(filepath).expect("failed to canonicalize path"));
    write_html().await;
    if cmd.open {
        open(&htmlpath);
    }
    Ok(())
}
