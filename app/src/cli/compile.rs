use std::fs;
use std::path::{Path, PathBuf};

use polarity_lang_driver::{Database, IR_PATH, JS_PATH};
use polarity_lang_printer::{Print, PrintCfg};

#[derive(clap::Args)]
pub struct Args {
    #[clap(value_parser, value_name = "FILE")]
    filepath: PathBuf,
}

pub async fn exec(cmd: Args) -> Result<(), Vec<miette::Report>> {
    let mut db = Database::from_path(&cmd.filepath);

    generate_ir(&mut db, &cmd.filepath).await?;
    generate_js(&mut db, &cmd.filepath).await?;

    Ok(())
}

async fn generate_ir(db: &mut Database, filepath: &Path) -> Result<(), Vec<miette::Report>> {
    let uri = db.resolve_path(filepath).map_err(|e| vec![e.into()])?;
    let ir = db.ir(&uri).await.map_err(|errs| db.pretty_errors(&uri, errs))?;

    let ir_path = target_path(IR_PATH, filepath, "ir");
    let mut file = fs::File::create(&ir_path).expect("Failed to create file");
    let cfg = PrintCfg::default();
    ir.print_io(&cfg, &mut file).expect("Failed to print to file");

    Ok(())
}

async fn generate_js(db: &mut Database, filepath: &Path) -> Result<(), Vec<miette::Report>> {
    let uri = db.resolve_path(filepath).map_err(|e| vec![e.into()])?;

    let js_path = target_path(JS_PATH, filepath, "js");
    let mut file = fs::File::create(&js_path).expect("Failed to create file");

    db.js(&uri, &mut file).await.map_err(|errs| db.pretty_errors(&uri, errs))?;

    Ok(())
}

fn target_path(base: &str, filepath: &Path, ext: &str) -> PathBuf {
    let base = Path::new(base);

    if !base.exists() {
        fs::create_dir_all(base).unwrap_or_else(|e| panic!("Failed to create directory: {e}"));
    }

    let mut path = Path::new(base).join(filepath.file_name().unwrap().to_string_lossy().as_ref());
    path.set_extension(ext);
    path
}
