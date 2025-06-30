use std::fmt;
use std::fs;
use std::io;
use std::path::PathBuf;

use driver::Database;
use printer::{Print, PrintCfg};

const LATEX_END: &str = r"\end{alltt}
";

fn latex_start(fontsize: &FontSize) -> String {
    let mut latex_start_string = "".to_string();
    latex_start_string.push_str("\\begin{alltt}\n");
    latex_start_string.push_str(&format!("\\{fontsize}"));
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
    #[clap(long, num_args = 0)]
    omit_lambda_sugar: bool,
    #[clap(long, num_args = 0)]
    omit_function_sugar: bool,
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

pub async fn exec(cmd: Args) -> miette::Result<()> {
    let mut db = Database::from_path(&cmd.filepath);
    let uri = db.resolve_path(&cmd.filepath)?;
    let prg = db.ust(&uri).await.map_err(|err| db.pretty_error(&uri, err))?;

    let mut stream: Box<dyn io::Write> = compute_output_stream(&cmd);

    let cfg = PrintCfg {
        width: cmd.width,
        latex: true,
        omit_decl_sep: true,
        de_bruijn: false,
        indent: cmd.indent,
        print_lambda_sugar: !cmd.omit_lambda_sugar,
        print_function_sugar: !cmd.omit_function_sugar,
        print_metavar_ids: false,
        print_metavar_args: false,
    };

    stream.write_all(latex_start(&cmd.fontsize).as_bytes()).unwrap();
    print_prg(&prg, &cfg, &mut stream);
    stream.write_all(LATEX_END.as_bytes()).unwrap();
    Ok(())
}

fn print_prg<W: io::Write>(prg: &ast::Module, cfg: &PrintCfg, stream: &mut W) {
    prg.print_latex(cfg, stream).expect("Failed to print to stdout");
    println!();
}
