use std::fs;
use std::io;
use std::io::prelude::*;
use std::path::Path;
use std::path::PathBuf;

use opener;

use driver::paths::CSS_PATH;
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

pub async fn write_html(filepath: &PathBuf, htmlpath: &PathBuf) {
    let mut db = Database::from_path(filepath);
    let uri = db.resolve_path(filepath).expect("Failed to resolve path");
    let prg = db.ust(&uri).await.expect("Failed to get UST");
    let cfg = PrintCfg::default();

    if !Path::new(CSS_PATH).exists() {
        write_css();
    }
    let mut stream = Box::new(fs::File::create(htmlpath).expect("Failed to create file"));
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

pub fn write_css() {
    let css_content = "
    body {
        font-family: 'Courier New', Courier, monospace;
        background-color: #000;
        color: #fff;
        margin: 0;
        padding: 20px;
        display: flex;
        justify-content: center;
        align-items: center;
        height: 100vh;
    }

    h1 {
        font-size: 2.5em;
        text-align: center;
        color: #fff;
        margin-bottom: 20px;
    }

    pre {
        background-color: #111;
        color: #f8f8f2;
        padding: 20px;
        border-radius: 8px;
        width: 80%;
        overflow-x: auto;
        font-size: 1.1em;
        line-height: 1.6;
        box-shadow: 0 4px 10px rgba(0, 0, 0, 0.5);
    }

    code {
        font-family: 'Courier New', Courier, monospace;
    }

    /* Syntax Highlighting Styling */
    .keyword {
        color: #ff79c6;
        font-weight: bold;
    }

    .type,
    .title {
        color: #8be9fd;
    }

    .dtor {
        color: #ff5555;
    }

    .ctor {
        color: #ff79c6;
    }

    .string {
        color: #f1fa8c;
    }

    .comment {
        color: #6272a4;
        font-style: italic;
    }
    ";
    fs::create_dir_all(Path::new(CSS_PATH).parent().unwrap())
        .expect("Failed to create directories");
    fs::write(CSS_PATH, css_content).expect("Failed to write CSS file");
}
