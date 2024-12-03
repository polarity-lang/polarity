use std::fs;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use askama::Template;
use opener;

use driver::paths::{CSS_PATH, CSS_TEMPLATE_PATH};
use driver::Database;
use printer::{Print, PrintCfg};

pub async fn write_html(filepath: &PathBuf, htmlpath: &PathBuf) {
    let mut db = Database::from_path(filepath);
    let uri = db.resolve_path(filepath).expect("Failed to resolve path");
    let prg = db.ust(&uri).await.expect("Failed to get UST");
    let cfg = PrintCfg::default();

    if !Path::new(CSS_PATH).exists() {
        fs::create_dir_all(Path::new(CSS_PATH).parent().unwrap())
            .expect("Failed to create CSS directory");
        fs::write(CSS_PATH, CSS_TEMPLATE_PATH).expect("Failed to create CSS file");
    }

    let mut stream = fs::File::create(htmlpath).expect("Failed to create file");

    let code = prg.print_html_to_string(Some(&cfg));
    let title = filepath.file_name().unwrap().to_str().unwrap();
    let output = generate_html(title, &code);

    stream.write_all(output.as_bytes()).expect("Failed to write to file");
    println!("new Generate: {}", prg.generate_docs())
}

pub fn open(filepath: &PathBuf) {
    let absolute_path = fs::canonicalize(filepath).expect("Failed to get absolute path");
    opener::open(&absolute_path).unwrap();
}

#[derive(Template)]
#[template(path = "module.html", escape = "none")]
struct ModuleTemplate<'a> {
    title: &'a str,
    code: &'a str,
}

fn generate_html(title: &str, code: &str) -> String {
    let template = ModuleTemplate { title, code };
    template.render().unwrap()
}
