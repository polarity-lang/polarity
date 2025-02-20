use std::{fs, path::{Path, PathBuf}};
use driver::DOCS_PATH;

pub fn get_target_path(path: &Path) -> PathBuf {
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
        //please note that this removes //?/ from windows paths
        let canonical_path = trim_windows_path_prefix(&canonical_path);
        print!("{:?}", canonical_path);
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

pub fn trim_windows_path_prefix(path: &Path) -> String {
    let canonical_str = path.to_string_lossy();
    if canonical_str.starts_with(r"\\?\") {
        canonical_str[4..].to_string()
    } else {
        canonical_str.to_string()
    }
}