use std::fs;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use askama::Template;
use opener;

use driver::paths::{CSS_PATH, CSS_TEMPLATE_PATH};
use driver::Database;

use crate::generate_docs::GenerateDocs;
use crate::json;

pub async fn write_html() {
    if !Path::new(CSS_PATH).exists() {
        fs::create_dir_all(Path::new(CSS_PATH).parent().unwrap())
            .expect("Failed to create CSS directory");
        fs::write(CSS_PATH, CSS_TEMPLATE_PATH).expect("Failed to create CSS file");
    }
    write_modules().await;
}

async fn write_modules() {
    let entries = json::read_json(Path::new("lang/docs/src/index.json"));
    let list = json::list_to_html(&entries);
    let mut all_modules = String::new();
    for entry in entries {
        let mut db = Database::from_path(json::get_path(&entry));
        let uri = db.resolve_path(json::get_path(&entry)).expect("Failed to resolve path");
        let prg = db.ust(&uri).await.expect("Failed to get UST");

        let code = prg.generate_docs();
        let content = generate_module(&json::get_name(&entry), &code);
        let output_file = generate_html(&json::get_name(&entry), &list, &content);

        let target_path = Path::new("target_pol/docs/").join(json::get_target(&entry));
        print!("Writing to file: {:?}", target_path.to_str().unwrap());
        let mut stream = fs::File::create(target_path).expect("Failed to create file");

        stream.write_all(output_file.as_bytes()).expect("Failed to write to file");

        all_modules.push_str(&output_file);
    }
}

pub fn open(filepath: &PathBuf) {
    let absolute_path = fs::canonicalize(filepath).expect("Failed to get absolute path");
    opener::open(&absolute_path).unwrap();
}

fn generate_module(title: &str, content: &str) -> String {
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