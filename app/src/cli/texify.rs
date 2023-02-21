use std::fs;
use std::io;
use std::io::Write;
use std::path::PathBuf;

use printer::latex::LatexWriter;
use printer::WriteColor;
use printer::{PrintCfg, PrintExt};
use source::Database;
use syntax::ust;

use super::ignore_colors::IgnoreColors;
use crate::result::IOError;

const LATEX_END: &str = r#"\end{alltt}
"#;

fn latex_start(fontsize: &Option<FontSize>) -> String {
    use FontSize::*;
    let latex_fontsize = match *fontsize {
        None => "\\scriptsize",
        Some(Tiny) => "\\tiny",
        Some(Scriptsize) => "\\scriptsize",
        Some(Footnotesize) => "\\footnotesize",
        Some(Small) => "\\small",
        Some(Normalsize) => "\\normalsize",
        Some(Large) => "\\large",
    };
    let mut latex_start_string = "".to_string();
    latex_start_string.push_str("\\begin{alltt}\n");
    latex_start_string.push_str(latex_fontsize);
    latex_start_string.push_str("\\ttfamily");
    latex_start_string
}

#[derive(clap::ValueEnum, Clone)]
pub enum FontSize {
    Tiny,
    Scriptsize,
    Footnotesize,
    Small,
    Normalsize,
    Large,
}

#[derive(clap::Args)]
pub struct Args {
    #[clap(value_parser, value_name = "FILE")]
    filepath: PathBuf,
    #[clap(long)]
    width: Option<usize>,
    #[clap(long)]
    fontsize: Option<FontSize>,
    #[clap(short, long, value_name = "FILE")]
    output: Option<PathBuf>,
}

/// Compute the output stream for the "texify" subcommand.
/// If an output filepath is specified, then that filepath is used.
/// Otherwise, the file extension is replaced by the `.tex` file extension.
fn compute_output_stream(cmd: &Args) -> Box<dyn io::Write> {
    match &cmd.output {
        Some(path) => Box::new(fs::File::create(path).expect("Failed to create file")),
        None => {
            let mut fp = cmd.filepath.clone();
            fp.set_extension("tex");
            Box::new(fs::File::create(fp).expect("Failed to create file"))
        }
    }
}

pub fn exec(cmd: Args) -> miette::Result<()> {
    let mut db = Database::default();
    let file =
        source::File::read(&cmd.filepath).map_err(IOError::from).map_err(miette::Report::from)?;
    let view = db.add(file).query();

    let prg = view.ust().map_err(|err| view.pretty_error(err))?;

    let mut stream: Box<dyn io::Write> = compute_output_stream(&cmd);

    let cfg = PrintCfg {
        width: cmd.width.unwrap_or(80),
        braces: ("\\{", "\\}"),
        omit_decl_sep: true,
        de_bruijn: false,
        indent: 4,
    };

    stream.write_all(latex_start(&cmd.fontsize).as_bytes()).unwrap();
    let mut stream = IgnoreColors::new(stream);
    let mut stream = LatexWriter::new(&mut stream);
    print_prg(prg, &cfg, &mut stream);
    stream.write_all(LATEX_END.as_bytes()).unwrap();
    Ok(())
}

fn print_prg<W: WriteColor>(prg: ust::Prg, cfg: &PrintCfg, stream: &mut W) {
    prg.print_colored(cfg, stream).expect("Failed to print to stdout");
    println!();
}
