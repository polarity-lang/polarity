use std::fs;
use std::io;
use std::path::PathBuf;
use std::io::prelude::*;
use opener;

use driver::Database;
use printer::{Print, PrintCfg};

const HTML_END: &str = " </code></pre>
    </div></body></html>";
const DOCS_PATH: &str = "target_pol/docs/";

fn html_start(cmd: &Args) -> String {
    let html = format!("<!DOCTYPE html>
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
        <pre><code>", filename = cmd.filepath.file_name().unwrap().to_string_lossy()).to_string();
    html
} 

#[derive(clap::Args)]
pub struct Args {
    #[clap(value_parser, value_name = "FILE")]
    filepath: PathBuf,
    #[clap(long, default_value_t = 80)]
    width: usize,
    #[clap(long, num_args = 0)]
    omit_lambda_sugar: bool,
    #[clap(long, num_args = 0)]
    omit_function_sugar: bool,
    #[clap(long, default_value_t = 4)]
    indent: isize,
    #[clap(long, num_args = 0)]
    open: bool,
}

fn compute_output_stream(path: &PathBuf) -> Box<dyn io::Write> {
        Box::new(fs::File::create(path).expect("Failed to create file"))
    }

pub fn exec(cmd: Args) -> miette::Result<()> {
    let mut db = Database::from_path(&cmd.filepath);
    let uri = db.resolve_path(&cmd.filepath)?;
    let prg = db.ust(&uri).map_err(|err| db.pretty_error(&uri, err))?;

    let mut stream: Box<dyn io::Write> = compute_output_stream(&get_output_path(&cmd));

    let cfg = PrintCfg {
        width: cmd.width,
        latex: false,
        omit_decl_sep: true,
        de_bruijn: false,
        indent: cmd.indent,
        print_lambda_sugar: !cmd.omit_lambda_sugar,
        print_function_sugar: !cmd.omit_function_sugar,
        print_metavar_ids: false,
    };

    stream.write_all(html_start(&cmd).as_bytes()).unwrap();
    print_prg(&prg, &cfg, &mut stream);
    stream.write_all(HTML_END.as_bytes()).unwrap();
    if cmd.open {
        open(&cmd);
    }
    Ok(())
}


fn get_output_path(cmd: &Args) -> PathBuf {
            let path = format!("{}{}", DOCS_PATH, cmd.filepath.file_name().unwrap().to_string_lossy());
            let mut fp = PathBuf::from(path);
            fp.set_extension("html");
            fp
        }
    
fn print_prg<W: io::Write>(prg: &ast::Module, cfg: &PrintCfg, stream: &mut W) {
    prg.print_html(cfg, stream).expect("Failed to print to stdout");
    println!();
}

fn open(cmd: &Args){
    let path = get_output_path(cmd);
    let absolute_path = fs::canonicalize(&path).expect("Failed to get absolute path");
    opener::open(&absolute_path).unwrap();
}

