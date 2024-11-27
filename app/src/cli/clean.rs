use std::fs;

use driver::paths::TARGET_PATH;

pub async fn exec() -> miette::Result<()> {
    if std::path::Path::new(TARGET_PATH).exists() {
        fs::remove_dir_all(TARGET_PATH).map_err(miette::Report::msg)?;
    }
    Ok(())
}
