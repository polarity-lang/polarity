use std::fs;
use std::io;
use std::path::PathBuf;

use printer::{Print, PrintCfg};
use query::Database;

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
    let mut db = Database::from_path(&cmd.filepath);
    let uri = db.resolve_path(&cmd.filepath)?;
    let prg = db.lift(&uri, &cmd.r#type).map_err(miette::Report::msg)?;

    // Write to file or to stdout
    let mut stream: Box<dyn io::Write> = match cmd.output {
        Some(path) => Box::new(fs::File::create(path).expect("Failed to create file")),
        None => Box::new(io::stdout()),
    };

    print_prg(&prg, &PrintCfg::default(), &mut stream);

    Ok(())
}

fn print_prg<W: io::Write>(prg: &ast::Module, cfg: &PrintCfg, stream: &mut W) {
    prg.print_io(cfg, stream).expect("Failed to print to stdout");
    println!();
}
