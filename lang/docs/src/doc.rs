use std::fs;
use std::path::Path;

use askama::Template;

use polarity_lang_driver::Database;
use polarity_lang_driver::paths::{CSS_PATH, CSS_TEMPLATE_PATH};

use crate::generate_docs::GenerateDocs;
use crate::generate_html_from_paths;
use crate::util::{get_absolut_css_path, get_files};

pub async fn write_html() {
    if !Path::new(CSS_PATH).exists() {
        fs::create_dir_all(Path::new(CSS_PATH).parent().unwrap())
            .expect("Failed to create CSS directory");
        fs::write(CSS_PATH, CSS_TEMPLATE_PATH).expect("Failed to create CSS file");
    }
    write_modules().await;
}

async fn write_modules() {
    let css_path = get_absolut_css_path();
    let folders: Vec<&Path> = vec![Path::new("examples/"), Path::new("std")];
    let path_list = get_files(folders.clone());
    let list = generate_html_from_paths(folders);
    for (source_path, target_path) in path_list {
        let mut db = Database::from_path(&source_path);
        let uri = db.resolve_path(&source_path).expect("Failed to resolve path");
        let prg = db.ust(&uri).await.expect("Failed to get UST");

        let title = source_path.file_stem().unwrap().to_str().unwrap();
        let code = prg.generate_docs();
        let content = generate_module_docs(title, &code);
        let html_file = generate_html(title, &list, &content, &css_path);

        fs::write(target_path, html_file.as_bytes()).expect("Failed to write to file");
    }
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

fn generate_html(title: &str, list: &str, code: &str, css: &str) -> String {
    let template = IndexTemplate { title, list, code, css };
    template.render().unwrap()
}
#[derive(Template)]
#[template(path = "index.html", escape = "none")]
struct IndexTemplate<'a> {
    title: &'a str,
    list: &'a str,
    code: &'a str,
    css: &'a str,
}
