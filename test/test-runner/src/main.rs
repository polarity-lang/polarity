mod index;
mod phases;
mod runner;
mod suites;

use clap::Parser;

pub const TEST_SUITES_PATH: &str = "test/suites";
pub const EXAMPLES_PATH: &str = "examples";

/// Polarity Testsuite Runner
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[clap(long)]
    filter: Option<String>,
    #[clap(long, num_args = 0)]
    debug: bool,
    #[clap(long, num_args = 0)]
    update_expected: bool,
}

fn main() {
    env_logger::builder().format_timestamp(None).format_level(false).format_target(false).init();
    let args = Args::parse();
    let runner = runner::Runner::load(crate::TEST_SUITES_PATH, crate::EXAMPLES_PATH);
    let config =
        runner::Config { filter: args.filter.unwrap_or_else(|| "*".to_owned()), debug: args.debug };
    let res = runner.run(&config);
    if args.update_expected {
        res.update_expected();
        println!("Updated expected outputs.");
    } else {
        res.print();
    }
    if !res.success() {
        std::process::exit(1);
    }
}
