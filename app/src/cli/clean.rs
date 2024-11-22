use driver::paths::TARGET_PATH;
use std::fs;

pub fn exec() -> miette::Result<()> {
    if std::path::Path::new(TARGET_PATH).exists() {
        fs::remove_dir_all(TARGET_PATH).map_err(miette::Report::msg)?;
    }
    Ok(())
}
