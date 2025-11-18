use url::Url;

use crate::paths;

#[cfg(not(target_arch = "wasm32"))]
pub fn locate_std() -> Option<Url> {
    let cwd = std::env::current_dir().ok()?;
    let std_path = cwd.join(paths::STD_PATH);
    if !std_path.is_dir() {
        return None;
    }
    let uri = Url::from_file_path(std_path).ok()?;
    Some(uri)
}

#[cfg(target_arch = "wasm32")]
pub fn locate_std() -> Option<Url> {
    Some(Url::parse(&format!("file:///{}", paths::STD_PATH)).ok()?)
}
