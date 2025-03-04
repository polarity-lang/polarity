use std::fs::File;
use std::path::PathBuf;

use driver::Database;
use printer::{ColorChoice, Print, PrintCfg, StandardStream, WriteColor};

use crate::utils::ignore_colors::IgnoreColors;

#[derive(clap::Args)]
pub struct Args {
    #[clap(value_parser, value_name = "FILE")]
    filepath: PathBuf,
    #[clap(long)]
    width: Option<usize>,
    #[clap(long, num_args = 0)]
    omit_lambda_sugar: bool,
    #[clap(long, num_args = 0)]
    omit_function_sugar: bool,
    #[clap(long, num_args = 0)]
    inplace: bool,
    #[clap(long, default_value_t = 4)]
    indent: isize,
    #[clap(short, long, value_name = "FILE")]
    output: Option<PathBuf>,
    #[clap(long, num_args = 0)]
    de_bruijn: bool,
    /// Print the typechecked instead of renamed syntax tree
    #[clap(long, num_args = 0)]
    checked: bool,
}

/// Compute the output stream for the "fmt" subcommand.
/// If an output filepath is specified, then that filepath is used.
/// Otherwise, the formatted output is printed on the terminal.
/// If the --inplace flag is specified, then the input file is overwritten.
fn compute_output_stream(cmd: &Args) -> Box<dyn WriteColor> {
    if cmd.inplace {
        return Box::new(IgnoreColors::new(
            File::create(cmd.filepath.clone()).expect("Failed to create file"),
        ));
    }
    match &cmd.output {
        Some(path) => {
            Box::new(IgnoreColors::new(File::create(path).expect("Failed to create file")))
        }
        None => Box::new(StandardStream::stdout(ColorChoice::Auto)),
    }
}

pub async fn exec(cmd: Args) -> miette::Result<()> {
    let mut db = Database::from_path(&cmd.filepath);
    let uri = db.resolve_path(&cmd.filepath)?;
    let prg = if cmd.checked { db.ast(&uri).await } else { db.ust(&uri).await }
        .map_err(|err| db.pretty_error(&uri, err))?;

    // Write to file or to stdout
    let mut stream: Box<dyn WriteColor> = compute_output_stream(&cmd);

    let cfg = PrintCfg {
        width: cmd.width.unwrap_or_else(terminal_width),
        latex: false,
        omit_decl_sep: false,
        de_bruijn: cmd.de_bruijn,
        indent: cmd.indent,
        print_lambda_sugar: !cmd.omit_lambda_sugar,
        print_function_sugar: !cmd.omit_function_sugar,
        print_metavar_ids: false,
        print_metavar_args: false,
    };

    print_prg(&prg, &cfg, &mut stream);

    Ok(())
}

fn print_prg<W: WriteColor>(prg: &ast::Module, cfg: &PrintCfg, stream: &mut W) {
    prg.print_colored(cfg, stream).expect("Failed to print to stdout");
    println!();
}

fn terminal_width() -> usize {
    termsize::get().map(|size| size.cols as usize).unwrap_or(printer::DEFAULT_WIDTH)
}
