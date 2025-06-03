use std::fs;
use std::path::{Path, PathBuf};

use driver::{Database, IR_PATH};
use printer::{Print, PrintCfg};

#[derive(clap::Args)]
pub struct Args {
    #[clap(value_parser, value_name = "FILE")]
    filepath: PathBuf,
}

pub async fn exec(cmd: Args) -> miette::Result<()> {
    let mut db = Database::from_path(&cmd.filepath);

    generate_ir(&mut db, &cmd.filepath).await?;
    generate_js(&mut db, &cmd.filepath).await?;

    Ok(())
}

async fn generate_ir(db: &mut Database, filepath: &Path) -> miette::Result<()> {
    let uri = db.resolve_path(&filepath)?;
    let ir = db.ir(&uri).await.map_err(|err| db.pretty_error(&uri, err))?;

    if !Path::new(IR_PATH).exists() {
        fs::create_dir_all(IR_PATH).expect("Failed to create IR directory");
    }

    let ir_path = target_path(filepath, "ir");
    let mut file = fs::File::create(&ir_path).expect("Failed to create file");
    let cfg = PrintCfg::default();
    ir.print_io(&cfg, &mut file).expect("Failed to print to file");

    Ok(())
}

async fn generate_js(db: &mut Database, filepath: &Path) -> miette::Result<()> {
    let uri = db.resolve_path(&filepath)?;

    if !Path::new(IR_PATH).exists() {
        fs::create_dir_all(IR_PATH).expect("Failed to create IR directory");
    }

    let js_path = target_path(filepath, "js");
    let mut file = fs::File::create(&js_path).expect("Failed to create file");

    db.js(&uri, &mut file).await.map_err(|err| db.pretty_error(&uri, err))?;

    Ok(())
}

fn target_path(filepath: &Path, ext: &str) -> PathBuf {
    let mut path =
        Path::new(IR_PATH).join(filepath.file_name().unwrap().to_string_lossy().as_ref());
    path.set_extension(ext);
    path
}
