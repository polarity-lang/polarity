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
    let htmlpath = Path::new(DOCS_PATH).join("index.html");
    write_html(filepath).await;
    if cmd.open {
        open(&htmlpath);
    }
    Ok(())
}
