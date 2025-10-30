use std::{fs, path::PathBuf};

use polarity_lang_parser::lexer::Lexer;

pub async fn exec(args: Args) -> Result<(), Vec<miette::Report>> {
    let src = fs::read_to_string(&args.filepath).expect("Failed to read file");
    let lexer = Lexer::new(&src);
    for tok in lexer {
        match tok {
            Ok((p1, tok, p2)) => println!("{tok} at ({p1},{p2})"),
            Err(err) => println!("{err}"),
        }
    }
    Ok(())
}

#[derive(clap::Args)]
pub struct Args {
    #[clap(value_parser, value_name = "FILE")]
    filepath: PathBuf,
}
