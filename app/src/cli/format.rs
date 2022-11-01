use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use printer::latex::LatexWriter;
use printer::{ColorChoice, PrintCfg, PrintExt, StandardStream, WriteColor};
use source::Database;
use syntax::ust;

use crate::result::IOError;

use super::ignore_colors::IgnoreColors;

const LATEX_START: &str = r#"\begin{alltt}
\footnotesize
\ttfamily
"#;
const LATEX_END: &str = r#"\end{alltt}
"#;

#[derive(clap::Args)]
pub struct Args {
    #[clap(value_parser, value_name = "FILE")]
    filepath: PathBuf,
    #[clap(long, num_args = 0)]
    latex: bool,
    #[clap(long)]
    width: Option<usize>,
    #[clap(short, long, value_name = "FILE")]
    output: Option<PathBuf>,
}

pub fn exec(cmd: Args) -> miette::Result<()> {
    let mut db = Database::default();
    let file =
        source::File::read(&cmd.filepath).map_err(IOError::from).map_err(miette::Report::from)?;
    let view = db.add(file).query();

    let prg = view.ust().map_err(|err| view.pretty_error(err))?;

    let width = cmd.width.unwrap_or_else(terminal_width);

    // Write to file or to stdout
    let mut stream: Box<dyn WriteColor> = match cmd.output {
        Some(path) => {
            Box::new(IgnoreColors::new(File::create(path).expect("Failed to create file")))
        }
        None => Box::new(StandardStream::stdout(ColorChoice::Auto)),
    };

    let cfg = PrintCfg { width, braces: if cmd.latex { ("\\{", "\\}") } else { ("{", "}") } };
    if cmd.latex {
        stream.write_all(LATEX_START.as_bytes()).unwrap();
        {
            let mut stream = LatexWriter::new(&mut stream);
            print_prg(prg, &cfg, &mut stream)
        }
        stream.write_all(LATEX_END.as_bytes()).unwrap();
    } else {
        print_prg(prg, &cfg, &mut stream)
    }

    Ok(())
}

fn print_prg<W: WriteColor>(prg: ust::Prg, cfg: &PrintCfg, stream: &mut W) {
    prg.print_colored(cfg, stream).expect("Failed to print to stdout");
    println!();
}

fn terminal_width() -> usize {
    termsize::get().map(|size| size.cols as usize).unwrap_or(printer::DEFAULT_WIDTH)
}
