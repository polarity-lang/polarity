use std::fs;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use askama::Template;
use opener;

use driver::paths::{CSS_PATH, CSS_TEMPLATE_PATH, EXAMPLE_PATH};
use driver::Database;

use crate::generate_docs::GenerateDocs;

pub async fn write_html(filepath: &PathBuf, htmlpath: &PathBuf) {
    let example_path = Path::new(EXAMPLE_PATH);
    let mut db = Database::from_path(filepath);
    let uri = db.resolve_path(filepath).expect("Failed to resolve path");
    let prg = db.ust(&uri).await.expect("Failed to get UST");

    if !Path::new(CSS_PATH).exists() {
        fs::create_dir_all(Path::new(CSS_PATH).parent().unwrap())
            .expect("Failed to create CSS directory");
        fs::write(CSS_PATH, CSS_TEMPLATE_PATH).expect("Failed to create CSS file");
    }

    let mut stream = fs::File::create(htmlpath).expect("Failed to create file");

    let code = prg.generate_docs();
    let title = filepath.file_name().unwrap().to_str().unwrap();
    let list = make_list(get_files(example_path));
    let content = generate_module(title, &code);
    let output = generate_html(title, &list, &content);

    stream.write_all(output.as_bytes()).expect("Failed to write to file");
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
}

fn generate_html(title: &str, list: &str, code: &str) -> String {
    let template = IndexTemplate { title, list, code };
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

fn get_files(path: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in fs::read_dir(path).expect("Failed to read directory") {
        let entry = entry.expect("Failed to get entry");
        let path = entry.path();
        if path.is_file() {
            files.push(path);
        }
    }
    files
}

fn make_list(files: Vec<PathBuf>) -> String {
    let mut list = String::new();
    for file in files {
        let name = file.file_name().unwrap().to_str().unwrap();
        list.push_str(&format!("<li><a onclick=\"showContent('{}')\">{}</a></li>", name, name));
    }
    list
}