use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use driver::{Database, IR_PATH};

#[derive(clap::Args)]
pub struct Args {
    #[clap(value_parser, value_name = "FILE")]
    filepath: PathBuf,
}

pub async fn exec(cmd: Args) -> miette::Result<()> {
    let mut db = Database::from_path(&cmd.filepath);
    let uri = db.resolve_path(&cmd.filepath)?;
    let ir = db.ir(&uri).await.map_err(|err| db.pretty_error(&uri, err))?;

    if !Path::new(IR_PATH).exists() {
        fs::create_dir_all(IR_PATH).expect("Failed to create IR directory");
    }

    let ir_path = target_path(&cmd.filepath);
    let mut file = fs::File::create(&ir_path).expect("Failed to create file");
    write!(&mut file, "{:#?}", ir).expect("Failed to write to file");

    Ok(())
}

fn target_path(filepath: &Path) -> PathBuf {
    let mut path =
        Path::new(IR_PATH).join(filepath.file_name().unwrap().to_string_lossy().as_ref());
    path.set_extension("ir");
    path
}
