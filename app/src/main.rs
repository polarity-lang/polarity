mod cli;
mod result;
mod rt;

pub const VERSION: &str = env!("VERSION");

fn main() {
    cli::exec();
}
