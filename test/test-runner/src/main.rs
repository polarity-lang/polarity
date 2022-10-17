mod cases;
mod cli;
mod index;
mod infallible;
mod phases;
mod runner;
mod suites;

pub const TEST_SUITES_PATH: &str = "test/suites";
pub const EXAMPLES_PATH: &str = "examples";

fn main() {
    cli::exec()
}
