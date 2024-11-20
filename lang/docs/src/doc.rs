use opener;
use std::fs;
use std::io;
use std::io::prelude::*;
use std::path::Path;
use std::path::PathBuf;

use driver::Database;
use printer::{Print, PrintCfg};

const HTML_END: &str = " </code></pre>
    </div></body></html>";

fn html_start(filepath: &Path) -> String {
    format!(
        "<!DOCTYPE html>
<html lang=\"en\">
<head>
    <meta charset=\"UTF-8\">
    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">
    <title>{filename}</title>
    <link rel=\"stylesheet\" href=\"style.css\">
</head>
<body>
<div>
        <h1>{filename}</h1>
        <pre><code>",
        filename = filepath.file_name().unwrap().to_string_lossy()
    )
}

pub fn write_html(filepath: &PathBuf, htmlpath: &PathBuf) {
    let mut db = Database::from_path(filepath);
    let uri = db.resolve_path(filepath).expect("Failed to resolve path");
    let prg = db.ust(&uri).expect("Failed to get UST");
    let cfg = PrintCfg::default();
    let mut stream = Box::new(fs::File::create(htmlpath).expect("Failed to create file"));
    println!("{:?}", htmlpath);
    stream.write_all(html_start(filepath).as_bytes()).expect("Failed to write to file");
    print_prg(&prg, &cfg, &mut stream);
    stream.write_all(HTML_END.as_bytes()).expect("Failed to write to file");
}

fn print_prg<W: io::Write>(prg: &ast::Module, cfg: &PrintCfg, stream: &mut W) {
    prg.print_html(cfg, stream).expect("Failed to print to stdout");
    println!();
}

pub fn open(filepath: &PathBuf) {
    let absolute_path = fs::canonicalize(filepath).expect("Failed to get absolute path");
    opener::open(&absolute_path).unwrap();
}
