use std::fs;
use std::path::{Path, PathBuf};

use polarity_lang_driver::{Database, IR_PATH};
use polarity_lang_printer::{Print, PrintCfg};

#[derive(clap::Args)]
pub struct Args {
    #[clap(value_parser, value_name = "FILE")]
    filepath: PathBuf,
}

pub async fn exec(cmd: Args) -> Result<(), Vec<miette::Report>> {
    let mut db = Database::from_path(&cmd.filepath);
    let uri = db.resolve_path(&cmd.filepath).map_err(|e| vec![e.into()])?;
    let ir = db.ir(&uri).await.map_err(|errs| db.pretty_errors(&uri, errs))?;

    if !Path::new(IR_PATH).exists() {
        fs::create_dir_all(IR_PATH).expect("Failed to create IR directory");
    }

    let ir_path = target_path(&cmd.filepath);
    let mut file = fs::File::create(&ir_path).expect("Failed to create file");
    let cfg = PrintCfg::default();
    ir.print_io(&cfg, &mut file).expect("Failed to print to file");

    Ok(())
}

fn target_path(filepath: &Path) -> PathBuf {
    let mut path =
        Path::new(IR_PATH).join(filepath.file_name().unwrap().to_string_lossy().as_ref());
    path.set_extension("ir");
    path
}
