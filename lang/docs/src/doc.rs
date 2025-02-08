use std::fs;
use std::path::{Path, PathBuf};

use askama::Template;
use opener;

use driver::paths::{CSS_PATH, CSS_TEMPLATE_PATH, DOCS_PATH};
use driver::Database;

use crate::generate_docs::GenerateDocs;

pub async fn write_html() {
    if !Path::new(CSS_PATH).exists() {
        fs::create_dir_all(Path::new(CSS_PATH).parent().unwrap())
            .expect("Failed to create CSS directory");
        fs::write(CSS_PATH, CSS_TEMPLATE_PATH).expect("Failed to create CSS file");
    }
    write_modules().await;
}

async fn write_modules() {
    let folders: Vec<&Path> =
        vec![Path::new("examples/"), Path::new("std")];
    let path_list = get_all_filepaths(folders);
    let list = list_to_html(&path_list);
    for path in path_list {
        let mut db = Database::from_path(&path);
        let uri = db.resolve_path(&path).expect("Failed to resolve path");
        let prg = db.ust(&uri).await.expect("Failed to get UST");

        let code = prg.generate_docs();
        let content = generate_module_docs(path.file_stem().unwrap().to_str().unwrap(), &code);
        let html_file = generate_html(path.file_stem().unwrap().to_str().unwrap(), &list, &content);

        let target_path = get_target_path(&path);
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent).expect("Failed to create directories");
        }

        fs::write(target_path, html_file.as_bytes()).expect("Failed to write to file");
    }
}

pub fn open(filepath: &PathBuf) {
    let absolute_path = fs::canonicalize(filepath).expect("Failed to get absolute path");
    opener::open(&absolute_path).unwrap();
}

fn generate_module_docs(title: &str, content: &str) -> String {
    let template = ModuleTemplate { title, content };
    template.render().unwrap()
}
#[derive(Template)]
#[template(path = "module.html", escape = "none")]
struct ModuleTemplate<'a> {
    title: &'a str,
    content: &'a str,
}

fn generate_html(title: &str, list: &str, code: &str) -> String {
    let template = IndexTemplate { title, list, code, start: title };
    template.render().unwrap()
}
#[derive(Template)]
#[template(path = "index.html", escape = "none")]
struct IndexTemplate<'a> {
    title: &'a str,
    list: &'a str,
    code: &'a str,
    start: &'a str,
}

fn get_filepaths_from_folder(folder: &Path) -> Vec<PathBuf> {
    let mut filepaths = Vec::new();
    if folder.is_dir() {
        for entry in fs::read_dir(folder).expect("Failed to read directory") {
            let entry = entry.expect("Failed to get directory entry");
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("pol") {
                filepaths.push(path);
            } else if path.is_dir() {
                filepaths.append(&mut get_filepaths_from_folder(&path));
            }
        }
    }
    filepaths
}

fn get_all_filepaths(folders: Vec<&Path>) -> Vec<PathBuf> {
    let mut all_filepaths = Vec::new();
    for folder in folders {
        let mut filepaths = get_filepaths_from_folder(folder);
        all_filepaths.append(&mut filepaths);
    }
    all_filepaths
}

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

pub fn list_to_html(list: &Vec<PathBuf>) -> String {
    let mut html = String::new();
    for path in list {
        html.push_str(&path_to_html(&get_target_path(path)));
    }
    html
}

fn path_to_html(path: &Path) -> String {
    format!(
        "<li><a href=../{}/{}>{}</a></li>",
        get_parent_folder(path),
        path.file_name().unwrap().to_string_lossy(),
        path.file_stem().unwrap().to_string_lossy()
    )
}

fn get_parent_folder(path: &Path) -> String {
    let parent = path.parent().expect("Failed to get parent directory");
    parent.file_stem().expect("Failed to get folder name").to_string_lossy().to_string()
}
