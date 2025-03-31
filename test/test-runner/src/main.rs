mod index;
mod phases;
mod runner;
mod suites;

use clap::Parser;

pub const TEST_SUITES_PATH: &str = "../suites";
pub const EXAMPLES_PATH: &str = "../../examples";
pub const STDLIB_PATH: &str = "../../std";

/// Polarity Testsuite Runner
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[clap(long)]
    filter: Option<String>,
    #[clap(long, num_args = 0)]
    update_expected: bool,
    /// Enable trace logging
    #[clap(long)]
    trace: bool,
    /// Enable debug logging
    #[clap(long)]
    debug: bool,
}

fn main() {
    let args = Args::parse();

    // Initialize the logger based on the flags
    let mut builder = env_logger::Builder::from_default_env();
    builder.format_timestamp(None).format_level(false).format_target(false);

    if args.trace {
        builder.filter_level(log::LevelFilter::Trace);
    } else if args.debug {
        builder.filter_level(log::LevelFilter::Debug);
    } else {
        builder.filter_level(log::LevelFilter::Info);
    }

    builder.init();

    let runner =
        runner::Runner::load(crate::TEST_SUITES_PATH, crate::EXAMPLES_PATH, crate::STDLIB_PATH);
    let mut res = runner.run(&args);
    if args.update_expected {
        let all_tests_fixed = res.update_expected();
        println!("Updated expected outputs.");

        // Return non-zero exit code if not all tests could be fixed
        if !all_tests_fixed {
            println!("Warning: There were tests of which expected output could not be updated, because they failed for a reason other than having an unexpected output.");
            std::process::exit(1);
        }
    } else {
        res.print();
        if !res.success() {
            std::process::exit(1);
        }
    }
}
