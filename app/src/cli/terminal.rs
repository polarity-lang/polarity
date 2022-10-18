use printer::{ColorChoice, PrintExt, StandardStream};
use syntax::ust;

pub fn print_prg(prg: ust::Prg) {
    printer::Alloc::new()
        .print_colored(&prg, width(), StandardStream::stdout(ColorChoice::Auto))
        .expect("Failed to print to stdout");
    println!();
}

pub fn width() -> usize {
    termsize::get().map(|size| size.cols as usize).unwrap_or(printer::DEFAULT_WIDTH)
}
