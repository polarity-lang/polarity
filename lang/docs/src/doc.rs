use std::fs;
use std::path::Path;

use askama::Template;

use driver::paths::{CSS_PATH, CSS_TEMPLATE_PATH};
use driver::Database;

use crate::generate_docs::GenerateDocs;
use crate::util::{get_files, generate_html_link_list};

pub async fn write_html() {
    if !Path::new(CSS_PATH).exists() {
        fs::create_dir_all(Path::new(CSS_PATH).parent().unwrap())
            .expect("Failed to create CSS directory");
        fs::write(CSS_PATH, CSS_TEMPLATE_PATH).expect("Failed to create CSS file");
    }
    write_modules().await;
}

async fn write_modules() {
    let folders: Vec<&Path> = vec![Path::new("examples/"), Path::new("std")];
    let path_list = get_files(folders);
    let list = generate_html_link_list(&path_list);
    for (source_path, target_path) in path_list {
        let mut db = Database::from_path(&source_path);
        let uri = db.resolve_path(&source_path).expect("Failed to resolve path");
        let prg = db.ust(&uri).await.expect("Failed to get UST");

        let code = prg.generate_docs();
        let content = generate_module_docs(source_path.file_stem().unwrap().to_str().unwrap(), &code);
        let html_file = generate_html(source_path.file_stem().unwrap().to_str().unwrap(), &list, &content);

        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent).expect("Failed to create directories");
        }

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

fn generate_html(title: &str, list: &str, code: &str) -> String {
    let template = IndexTemplate { title, list, code, start: title};
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
