use std::fmt;
use std::fs;
use std::io;
use std::path::PathBuf;

use polarity_lang_driver::Database;
use polarity_lang_printer::{Print, PrintCfg};

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

#[derive(clap::ValueEnum, Clone)]
enum Backend {
    Latex,
    Typst,
}

impl fmt::Display for Backend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Backend::*;
        match self {
            Latex => write!(f, "latex"),
            Typst => write!(f, "typst"),
        }
    }
}

#[derive(clap::Args)]
pub struct Args {
    #[clap(long, default_value_t=Backend::Latex)]
    backend: Backend,
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

/// Compute the output stream for the "prettify" subcommand.
/// If an output filepath is specified, then that filepath is used.
/// Otherwise, the file extension is replaced by the `.tex` or `.typ` file extension.
fn compute_output_stream(cmd: &Args) -> Box<dyn io::Write> {
    let file_extension = match cmd.backend {
        Backend::Latex => "tex",
        Backend::Typst => "typ",
    };
    match &cmd.output {
        Some(path) => Box::new(fs::File::create(path).expect("Failed to create file")),
        None => {
            let mut fp = cmd.filepath.clone();
            fp.set_extension(file_extension);
            Box::new(fs::File::create(fp).expect("Failed to create file"))
        }
    }
}

pub async fn exec(cmd: Args) -> Result<(), Vec<miette::Report>> {
    let mut db = Database::from_path(&cmd.filepath);
    let uri = db.resolve_path(&cmd.filepath).map_err(|e| vec![e.into()])?;
    let prg = db.ust(&uri).await.map_err(|errs| db.pretty_errors(&uri, errs))?;

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
        print_metavar_solutions: false,
    };

    match cmd.backend {
        Backend::Latex => {
            stream.write_all(latex_start(&cmd.fontsize).as_bytes()).unwrap();
            prg.print_latex(&cfg, &mut stream).expect("Failed to print to stdout");
            println!();
            stream.write_all(LATEX_END.as_bytes()).unwrap();
        }
        Backend::Typst => {
            prg.print_typst(&cfg, &mut stream).expect("Failed to print to stdout");
            println!();
        }
    }
    Ok(())
}
