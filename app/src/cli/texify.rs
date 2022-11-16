use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use printer::latex::LatexWriter;
use printer::{ColorChoice, PrintCfg, PrintExt, StandardStream, WriteColor};
use source::Database;
use syntax::ust;

use crate::result::IOError;

use super::ignore_colors::IgnoreColors;

const LATEX_END: &str = r#"\end{alltt}
"#;

fn latex_start(fontsize: &Option<FontSize>) -> String {
    use FontSize::*;
    let latex_fontsize = match *fontsize {
        None => "\\scriptsize\n",
        Some(Tiny) => "\\tiny\n",
        Some(Scriptsize) => "\\scriptsize\n",
        Some(Footnotesize) => "\\footnotesize\n",
        Some(Small) => "\\small\n",
        Some(Normalsize) => "\\normalsize\n",
        Some(Large) => "\\large\n",
    };
    let mut latex_start_string = "".to_string();
    latex_start_string.push_str("\\begin{alltt}\n");
    latex_start_string.push_str(latex_fontsize);
    latex_start_string.push_str("\\ttfamily\n");
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

pub fn exec(cmd: Args) -> miette::Result<()> {
    let mut db = Database::default();
    let file =
        source::File::read(&cmd.filepath).map_err(IOError::from).map_err(miette::Report::from)?;
    let view = db.add(file).query();

    let prg = view.ust().map_err(|err| view.pretty_error(err))?;

    // Write to file or to stdout
    let mut stream: Box<dyn WriteColor> = match cmd.output {
        Some(path) => {
            Box::new(IgnoreColors::new(File::create(path).expect("Failed to create file")))
        }
        None => Box::new(StandardStream::stdout(ColorChoice::Auto)),
    };

    let cfg =
        PrintCfg { width: cmd.width.unwrap_or(80), braces: ("\\{", "\\}"), omit_decl_sep: true };

    stream.write_all(latex_start(&cmd.fontsize).as_bytes()).unwrap();
    {
        let mut stream = LatexWriter::new(&mut stream);
        print_prg(prg, &cfg, &mut stream)
    }
    stream.write_all(LATEX_END.as_bytes()).unwrap();
    Ok(())
}

fn print_prg<W: WriteColor>(prg: ust::Prg, cfg: &PrintCfg, stream: &mut W) {
    prg.print_colored(cfg, stream).expect("Failed to print to stdout");
    println!();
}