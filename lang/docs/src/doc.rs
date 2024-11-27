use std::fs;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use askama::Template;
use html_escape::decode_html_entities;
use opener;

use driver::paths::{CSS_PATH, CSS_TEMPLATE_PATH};
use driver::Database;
use printer::{Print, PrintCfg};

pub fn write_html(filepath: &PathBuf, htmlpath: &PathBuf) {
    let mut db = Database::from_path(filepath);
    let uri = db.resolve_path(filepath).expect("Failed to resolve path");
    let prg = db.ust(&uri).expect("Failed to get UST");
    let cfg = PrintCfg::default();

    if !Path::new(CSS_PATH).exists() {
        let template_css_path = Path::new(CSS_TEMPLATE_PATH);
        if template_css_path.exists() {
            fs::copy(template_css_path, CSS_PATH).expect("Failed to copy CSS file");
        } else {
            eprintln!("Warning: template CSS file does not exist at {:?}", template_css_path);
        }
    }

    let mut stream = Box::new(fs::File::create(htmlpath).expect("Failed to create file"));
    let code = prg.print_html_to_string(Some(&cfg));

    let title = filepath.file_name().unwrap().to_str().unwrap();
    let output = generate_html(title, &code);
    let out = decode_html_entities(&output);
    stream.write_all(out.as_bytes()).expect("Failed to write to file");
}

pub fn open(filepath: &PathBuf) {
    let absolute_path = fs::canonicalize(filepath).expect("Failed to get absolute path");
    opener::open(&absolute_path).unwrap();
}

#[derive(Template)]
#[template(path = "code.html")]
struct HelloTemplate<'a> {
    title: &'a str,
    code: &'a str,
}

fn generate_html(title: &str, code: &str) -> String {
    let template = HelloTemplate { title, code };
    template.render().unwrap()
}
