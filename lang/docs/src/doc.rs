use std::fs;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use askama::Template;
use opener;

use driver::paths::{CSS_PATH, CSS_TEMPLATE_PATH, DOCS_PATH, EXAMPLE_PATH};
use driver::{Database, CODATA_PATH, DATA_PATH, STD_PATH};

use crate::generate_docs::GenerateDocs;

pub async fn write_html() {
    if !Path::new(CSS_PATH).exists() {
        fs::create_dir_all(Path::new(CSS_PATH).parent().unwrap())
            .expect("Failed to create CSS directory");
        fs::write(CSS_PATH, CSS_TEMPLATE_PATH).expect("Failed to create CSS file");
    }
    write_modules().await;
}

pub fn open(filepath: &PathBuf) {
    let absolute_path = fs::canonicalize(filepath).expect("Failed to get absolute path");
    opener::open(&absolute_path).unwrap();
}

#[derive(Template)]
#[template(path = "index.html", escape = "none")]
struct IndexTemplate<'a> {
    title: &'a str,
    list: &'a str,
    code: &'a str,
    start: &'a str,
}

fn generate_html(title: &str, list: &str, code: &str) -> String {
    let template = IndexTemplate { title, list, code, start: title };
    template.render().unwrap()
}

#[derive(Template)]
#[template(path = "module.html", escape = "none")]
struct ModuleTemplate<'a> {
    title: &'a str,
    content: &'a str,
}

fn generate_module(title: &str, content: &str) -> String {
    let template = ModuleTemplate { title, content };
    template.render().unwrap()
}

async fn write_modules() {
    let example_path = Path::new(EXAMPLE_PATH);
    let std_path = Path::new(STD_PATH);
    let codata_path = Path::new(CODATA_PATH);
    let data_path = Path::new(DATA_PATH);
    write(example_path).await;
    write(std_path).await;
    write(codata_path).await;
    write(data_path).await;
}

async fn write(path: &Path){
    let mut all_modules = String::new();
    let list = file_list(get_files(path));
    for file in get_files(path) {
        let mut db = Database::from_path(&file);
        let uri = db.resolve_path(&file).expect("Failed to resolve path");
        let prg = db.ust(&uri).await.expect("Failed to get UST");

        let title = file.file_stem().unwrap().to_str().unwrap();
        let title = title.replace('_', " ");
        let code = prg.generate_docs();
        let content = generate_module(&title, &code);
        let output_file = generate_html(&title, &list, &content);

        let htmlpath = get_path(&file);
        let mut stream = fs::File::create(htmlpath).expect("Failed to create file");

        stream.write_all(output_file.as_bytes()).expect("Failed to write to file");

        all_modules.push_str(&output_file);
    }
}

fn get_files(path: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if path.is_dir() {
        for entry in fs::read_dir(path).expect("Failed to read directory") {
            let entry = entry.expect("Failed to read entry");
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("pol") {
                files.push(path);
            }
        }
    }
    files
}

fn file_list(files: Vec<PathBuf>) -> String {
    let mut list = String::new();
    for file in files {
        let name = file.file_stem().unwrap().to_str().unwrap();
        let path = name.to_string() + ".html";
        let name = name.replace('_', " ");
        list.push_str(&format!("<li><a href={}>{}</a></li>", &path, name));
    }
    list
}

fn get_path(filepath: &Path) -> PathBuf {
    let mut path =
        Path::new(DOCS_PATH).join(filepath.file_name().unwrap().to_string_lossy().as_ref());
    path.set_extension("html");
    path
}
