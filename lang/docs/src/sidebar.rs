use std::fs;
use std::path::{Path, PathBuf};

use crate::util::{get_target_path, trim_windows_path_prefix};

#[derive(Debug)]
pub struct FileStructure {
    pub name: String,
    pub path: PathBuf,
}

impl FileStructure {
    pub fn new(name: String, path: PathBuf) -> Self {
        Self { name, path }
    }
}

#[derive(Debug)]
pub struct FolderStructure {
    pub name: String,
    pub path: PathBuf,
    pub files: Vec<FileStructure>,
    pub subfolders: Vec<FolderStructure>,
}

impl FolderStructure {
    fn new(name: String, path: PathBuf) -> Self {
        Self { name, files: Vec::new(), subfolders: Vec::new(), path }
    }

    fn add_file(&mut self, file: FileStructure) {
        self.files.push(file);
    }

    fn add_subfolder(&mut self, subfolder: FolderStructure) {
        self.subfolders.push(subfolder);
    }

    fn generate_html(&self) -> String {
        let mut html = String::new();
        html.push_str("<li class=\"folder\">");
        html.push_str(&format!("<span class=\"label\">{}</span>", self.name));
        if !self.files.is_empty() || !self.subfolders.is_empty() {
            html.push_str("<ul>");
            for file in &self.files {
                let canonical_path =
                    fs::canonicalize(&file.path).expect("Failed to canonicalize path");
                let target_path = get_target_path(&canonical_path);
                let trimed_path = trim_windows_path_prefix(&target_path);
                html.push_str(&format!(
                    "<li class=\"file\"><a href=\"{}\">{}</a></li>",
                    trimed_path, file.name
                ));
            }
            for subfolder in &self.subfolders {
                html.push_str(&subfolder.generate_html());
            }
            html.push_str("</ul>");
        }
        html.push_str("</li>");
        html
    }
}

fn build_folder_structure(path: &Path) -> FolderStructure {
    let mut folder_structure = FolderStructure::new(
        path.file_name().unwrap().to_string_lossy().to_string(),
        path.to_path_buf(),
    );

    if path.is_dir() {
        for entry in fs::read_dir(path).expect("Failed to read directory") {
            let entry = entry.expect("Failed to get directory entry");
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("pol") {
                folder_structure.add_file(FileStructure::new(
                    path.file_stem().unwrap().to_string_lossy().to_string(),
                    path.to_path_buf(),
                ));
            } else if path.is_dir() {
                folder_structure.add_subfolder(build_folder_structure(&path));
            }
        }
    }

    folder_structure
}

pub fn generate_html_from_paths(paths: Vec<&Path>) -> String {
    let mut html = String::new();
    html.push_str("<ul>");
    for path in paths {
        if path.is_dir() {
            html.push_str(&build_folder_structure(path).generate_html());
        }
    }
    html.push_str("</ul>");
    html
}
