use std::path::PathBuf;

// 1. --lib-path
// 2. POLARITY_LIB_PATHS environment variable (for non-standard packaging like nix, bundling etc.)
// 3. relative to the current working directory "./std" (usecase: when ran from polarity/ repo during development)
// 4. platform independent sensible installation path (xdg on Unix, use suitable library)
pub fn locate_libs(cli_paths: Option<Vec<PathBuf>>) -> Vec<PathBuf> {
    if let Some(cli_paths) = cli_paths {
        return cli_paths;
    }

    todo!()
}
