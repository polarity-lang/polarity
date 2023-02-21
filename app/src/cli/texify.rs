use std::fmt;
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

fn latex_start(fontsize: &FontSize) -> String {
    use FontSize::*;
    let latex_fontsize = match *fontsize {
        Tiny => "\\tiny",
        Scriptsize => "\\scriptsize",
        Footnotesize => "\\footnotesize",
        Small => "\\small",
        Normalsize => "\\normalsize",
        Large => "\\large",
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

impl fmt::Display for FontSize {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use FontSize::*;
        match self {
            Tiny => write!(f, "tiny"),
            Scriptsize => write!(f, "scriptsize"),
            Footnotesize => write!(f, "footnotesize"),
            Small => write!(f, "small"),
            Normalsize => write!(f, "normalsize"),
            Large => write!(f, "large"),
        }
    }
}

#[derive(clap::Args)]
pub struct Args {
    #[clap(value_parser, value_name = "FILE")]
    filepath: PathBuf,
    #[clap(long, default_value_t = 80)]
    width: usize,
    #[clap(long, default_value_t=FontSize::Scriptsize)]
    fontsize: FontSize,
    #[clap(long, default_value_t = 4)]
    indent: isize,
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
        width: cmd.width,
        braces: ("\\{", "\\}"),
        omit_decl_sep: true,
        de_bruijn: false,
        indent: cmd.indent,
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
