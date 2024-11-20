use docs::open;
use docs::write_html;
use std::path::PathBuf;

const DOCS_PATH: &str = "target_pol/docs/";

#[derive(clap::Args)]
pub struct Args {
    #[clap(value_parser, value_name = "FILE")]
    filepath: PathBuf,
    #[clap(long, num_args = 0)]
    open: bool,
}

pub fn exec(cmd: Args) -> miette::Result<()> {
    let htmlpath = get_path(&cmd);
    let filepath = &cmd.filepath;
    write_html(filepath, &htmlpath);
    if cmd.open {
        open(&htmlpath);
    }
    Ok(())
}

fn get_path(cmd: &Args) -> PathBuf {
    let path = format!("{}{}", DOCS_PATH, cmd.filepath.file_name().unwrap().to_string_lossy());
    let mut fp = PathBuf::from(path);
    fp.set_extension("html");
    fp
}
