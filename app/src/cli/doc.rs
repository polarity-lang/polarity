use std::path::Path;
use std::path::PathBuf;

use docs::open;
use docs::write_html;
use driver::paths::DOCS_PATH;

#[derive(clap::Args)]
pub struct Args {
    #[clap(value_parser, value_name = "FILE")]
    filepath: PathBuf,
    #[clap(long, num_args = 0)]
    open: bool,
}

pub async fn exec(cmd: Args) -> miette::Result<()> {
    let filepath = &cmd.filepath;
    let htmlpath = get_path(filepath);
    write_html().await;
    if cmd.open {
        open(&htmlpath);
    }
    Ok(())
}

fn get_path(filepath: &Path) -> PathBuf {
    let mut path =
        Path::new(DOCS_PATH).join(filepath.file_name().unwrap().to_string_lossy().as_ref());
    path.set_extension("html");
    path
}
