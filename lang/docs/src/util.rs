use driver::{CSS_PATH, DOCS_PATH};
use std::{
    fs,
    path::{Path, PathBuf},
};

/* pub fn get_target_path(path: &Path) -> PathBuf {
    let mut new_path = PathBuf::from(DOCS_PATH);
    new_path.push(get_parent_folder(path));
    if let Some(stem) = path.file_stem() {
        new_path.push(stem);
        new_path.set_extension("html");
    }
    if let Some(parent) = new_path.parent() {
        fs::create_dir_all(parent).expect("Failed to create directories");
    }
    new_path
} */

pub fn get_target_path(path: &Path) -> PathBuf {
    let path = fs::canonicalize(path).expect("Failed to canonicalize path");
    let mut components = path.components().peekable();
    let mut new_path = PathBuf::new();

    for component in components.by_ref() {
        new_path.push(component);
        if component.as_os_str() == "polarity" {
            new_path.push(DOCS_PATH);
            break;
        }
    }

    for component in components {
        new_path.push(component);
    }

    let stem = new_path.file_stem().map(|s| s.to_os_string());
    if let Some(stem) = stem {
        new_path.set_file_name(stem);
        new_path.set_extension("html");
    }

    new_path
}

pub fn get_files(folders: Vec<&Path>) -> Vec<(PathBuf, PathBuf)> {
    let mut pol_files = Vec::new();
    for folder in folders {
        if folder.is_dir() {
            for entry in fs::read_dir(folder).expect("Failed to read directory") {
                let entry = entry.expect("Failed to get directory entry");
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("pol") {
                    let target_path = get_target_path(&path);
                    create_parent_directory(&target_path);
                    fs::File::create(&target_path).expect("Failed to create file");
                    pol_files.push((path, target_path));
                } else if path.is_dir() {
                    pol_files.append(&mut get_files(vec![&path]));
                }
            }
        }
    }
    pol_files
}

pub fn generate_html_link_list(folders: &Vec<(PathBuf, PathBuf)>) -> String {
    let mut html = String::new();
    for (source_path, target_path) in folders {
        let canonical_path = fs::canonicalize(target_path).expect("Failed to canonicalize path");
        let canonical_path = trim_windows_path_prefix(&canonical_path);
        html.push_str(&format!(
            "<li><a href=\"{}\">{}</a></li>",
            canonical_path,
            source_path.file_stem().unwrap().to_string_lossy()
        ));
    }
    html
}

pub fn get_parent_folder(path: &Path) -> String {
    let parent = path.parent().expect("Failed to get parent directory");
    parent.file_stem().expect("Failed to get folder name").to_string_lossy().to_string()
}

pub fn open(filepath: &PathBuf) {
    let absolute_path = fs::canonicalize(filepath).expect("Failed to get absolute path");
    opener::open(&absolute_path).unwrap();
}

pub fn get_absolut_css_path() -> String {
    let css_path = fs::canonicalize(PathBuf::from(CSS_PATH)).expect("Failed to get absolute path");
    trim_windows_path_prefix(&css_path)
}

pub fn trim_windows_path_prefix(path: &Path) -> String {
    let canonical_str = path.to_string_lossy();
    canonical_str.strip_prefix(r"\\?\").unwrap_or(&canonical_str).to_string()
}

pub fn create_parent_directory(target_path: &Path) {
    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent).expect("Failed to create directories");
    }
}
