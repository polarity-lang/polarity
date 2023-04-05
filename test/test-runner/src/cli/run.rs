use crate::runner;

#[derive(clap::Args)]
pub struct Args {
    #[clap(long)]
    filter: Option<String>,
    #[clap(long, num_args = 0)]
    debug: bool,
    #[clap(long, num_args = 0)]
    update_expected: bool,
}

pub fn exec(cmd: Args) {
    let runner = runner::Runner::load(
        crate::TEST_SUITES_PATH,
        crate::EXAMPLES_PATH,
        crate::OOPSLA_EXAMPLES_PATH,
    );
    let config =
        runner::Config { filter: cmd.filter.unwrap_or_else(|| "*".to_owned()), debug: cmd.debug };
    let res = runner.run(&config);
    if cmd.update_expected {
        res.update_expected();
        println!("Updated expected outputs.");
    } else {
        res.print();
    }
    if !res.success() {
        std::process::exit(1);
    }
}
