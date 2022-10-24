use std::io::Write;
use std::path::PathBuf;

use printer::latex::LatexWriter;
use printer::{ColorChoice, PrintCfg, PrintExt, StandardStream, WriteColor};
use syntax::ust;

use crate::result::HandleErrorExt;

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
    #[clap(long, takes_value = false)]
    latex: bool,
    #[clap(long)]
    width: Option<usize>,
    #[clap(short, long, value_name = "FILE")]
    output: Option<PathBuf>,
}

pub fn exec(cmd: Args) {
    crate::rt::lower_filepath(&cmd.filepath).handle_with(|prg| {
        let width = cmd.width.unwrap_or_else(terminal_width);
        // TODO: Open file instead if output is given
        let mut stream = StandardStream::stdout(ColorChoice::Auto);
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
    })
}

fn print_prg<W: WriteColor>(prg: ust::Prg, cfg: &PrintCfg, stream: &mut W) {
    prg.print_colored(cfg, stream).expect("Failed to print to stdout");
    println!();
}

fn terminal_width() -> usize {
    termsize::get().map(|size| size.cols as usize).unwrap_or(printer::DEFAULT_WIDTH)
}
