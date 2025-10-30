use std::fs;

use polarity_lang_driver::paths::TARGET_PATH;

pub async fn exec() -> Result<(), Vec<miette::Report>> {
    if std::path::Path::new(TARGET_PATH).exists() {
        fs::remove_dir_all(TARGET_PATH).map_err(|e| vec![miette::Report::msg(e)])?;
    }
    Ok(())
}
