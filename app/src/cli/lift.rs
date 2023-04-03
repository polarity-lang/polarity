use std::fs;
use std::io;
use std::path::PathBuf;

use printer::{PrintCfg, PrintExt};
use source::Database;
use syntax::ust;

use crate::result::IOError;

#[derive(clap::Args)]
pub struct Args {
    #[clap(value_parser, value_name = "TYPE")]
    r#type: String,
    #[clap(value_parser, value_name = "FILE")]
    filepath: PathBuf,
    #[clap(short, long, value_name = "FILE")]
    output: Option<PathBuf>,
}

pub fn exec(cmd: Args) -> miette::Result<()> {
    let mut db = Database::default();
    let file =
        source::File::read(&cmd.filepath).map_err(IOError::from).map_err(miette::Report::from)?;
    let view = db.add(file).query();

    let prg = view.lift(&cmd.r#type).map_err(miette::Report::msg)?;

    // Write to file or to stdout
    let mut stream: Box<dyn io::Write> = match cmd.output {
        Some(path) => Box::new(fs::File::create(path).expect("Failed to create file")),
        None => Box::new(io::stdout()),
    };

    print_prg(prg, &PrintCfg { print_lambda_sugar: false, ..Default::default() }, &mut stream);

    Ok(())
}

fn print_prg<W: io::Write>(prg: ust::Prg, cfg: &PrintCfg, stream: &mut W) {
    prg.print(cfg, stream).expect("Failed to print to stdout");
    println!();
}
