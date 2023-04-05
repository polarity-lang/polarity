mod cases;
mod cli;
mod index;
mod infallible;
mod phases;
mod runner;
mod suites;

pub const TEST_SUITES_PATH: &str = "test/suites";
pub const EXAMPLES_PATH: &str = "examples";
pub const OOPSLA_EXAMPLES_PATH: &str = "oopsla_examples";

fn main() {
    cli::exec()
}
