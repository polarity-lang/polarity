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
    let htmlpath = get_path(&cmd);
    let filepath = &cmd.filepath;
    write_html(filepath, &htmlpath).await;
    if cmd.open {
        open(&htmlpath);
    }
    Ok(())
}

fn get_path(cmd: &Args) -> PathBuf {
    let mut path =
        Path::new(DOCS_PATH).join(cmd.filepath.file_name().unwrap().to_string_lossy().as_ref());
    path.set_extension("html");
    path
}
