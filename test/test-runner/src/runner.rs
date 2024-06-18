use std::path::Path;

use syntax::common::HashMap;
use url::Url;

use crate::Args;

use super::index::Index;
use super::phases::*;
use super::suites::{self, Case, Suite};

pub struct Runner {
    suites: HashMap<String, Suite>,
    /// An index of all the testsuites.
    /// This index is used for the `filter` commandline argument
    /// which can be used to filter the testcases which should be run.
    index: Index,
}

/// The default search string which is used to filter out testcases if
/// the `--filter` option was not passed on the command line.
const ALL_GLOB: &str = "*";

/// Create a search index for all the testsuites
fn create_index(suites: &HashMap<String, Suite>) -> Index {
    let mut index = Index::new();
    let mut writer = index.writer();

    for suite in suites.values() {
        for case in &suite.cases {
            let content = case.content().unwrap();
            writer.add(&suite.name, case, &content);
        }
    }

    writer.commit();
    index
}

impl Runner {
    pub fn load<P1, P2>(suites_path: P1, examples_path: P2) -> Self
    where
        P1: AsRef<Path>,
        P2: AsRef<Path>,
    {
        let mut suites: HashMap<_, _> =
            suites::load(suites_path.as_ref()).map(|suite| (suite.name.clone(), suite)).collect();
        suites.insert("examples".to_owned(), Suite::new(examples_path.as_ref().into()));

        let index = create_index(&suites);

        Self { suites, index }
    }

    /// Run all the testsuites and compute the combined result.
    pub fn run(&self, args: &Args) -> RunResult {
        let mut executed_cases: u32 = 0;
        let mut failed_cases: u32 = 0;
        let mut results: Vec<SuiteResult> = vec![];

        for suite in self.suites.values() {
            let result = self.run_suite(args, suite);

            executed_cases += result.executed_cases;
            failed_cases += result.failed_cases;
            results.push(result);
        }
        RunResult { results, executed_cases, failed_cases }
    }

    /// Run one individual testsuite
    pub fn run_suite(&self, args: &Args, suite: &suites::Suite) -> SuiteResult {
        // We first have to filter out those cases which should not be run.
        let search_string = match &args.filter {
            None => ALL_GLOB,
            Some(str) => str,
        };
        let matching_cases: Vec<Case> = self.index.searcher().search(search_string).collect();

        let mut executed_cases: u32 = 0;
        let mut failed_cases: u32 = 0;
        let mut results: Vec<CaseResult> = vec![];

        for case in &suite.cases {
            if !matching_cases.contains(case) {
                continue;
            }

            let case_result = self.run_case(&suite.config, case);

            executed_cases += 1;
            if case_result.result.is_err() {
                failed_cases += 1;
            }
            results.push(case_result);
        }
        SuiteResult { suite: suite.clone(), results, executed_cases, failed_cases }
    }

    /// Run one individual testcase within a testsuite
    pub fn run_case(&self, config: &suites::Config, case: &Case) -> CaseResult {
        let canonicalized_path = case.path.clone().canonicalize().unwrap();
        let uri = Url::from_file_path(canonicalized_path).unwrap();
        let input = (uri, case.content().unwrap());

        PartialRun::start(case.clone(), input)
            .then(config, Parse::new("parse"))
            .then(config, Lower::new("lower"))
            .then(config, Check::new("check"))
            .then(config, Print::new("print"))
            .then(config, Parse::new("reparse"))
            .then(config, Lower::new("relower"))
            .then(config, Check::new("recheck"))
            .report()
    }
}

// Run Result
//
//

/// The result of running all testsuites.
pub struct RunResult {
    /// The results of the individual testsuites.
    results: Vec<SuiteResult>,
    /// The number of cases that were executed for all testsuites combined.
    executed_cases: u32,
    /// The number of cases that failed in all testsuites combined.
    failed_cases: u32,
}

impl RunResult {
    pub fn success(&self) -> bool {
        self.failed_cases == 0
    }

    pub fn update_expected(&self) {
        for CaseResult { case, result } in self.case_results() {
            if let Err(Failure::Mismatch { ref actual, .. }) = result {
                case.set_expected(actual);
            }
        }
    }

    fn case_results(&self) -> impl Iterator<Item = &CaseResult> {
        self.results.iter().flat_map(|suite_res| suite_res.results.iter())
    }

    pub fn print(&self) {
        for suite in &self.results {
            suite.print()
        }
        println!(
            "In total: {}/{} successful",
            self.executed_cases - self.failed_cases,
            self.executed_cases
        );
    }
}

// Suite Result
//
//

/// The result of running one individual testsuite.
pub struct SuiteResult {
    /// The testsuite to which the result belongs.
    suite: Suite,
    /// The results of the individual testcases.
    results: Vec<CaseResult>,
    /// The number of cases that were executed for this testsuite.
    executed_cases: u32,
    /// The number of cases that failed in this testsuite.
    failed_cases: u32,
}

impl SuiteResult {
    pub fn print(&self) {
        let SuiteResult { suite, results, executed_cases, failed_cases } = self;
        println!("Suite \"{}\":", suite.name);
        results.iter().for_each(|x| x.print());
        println!("{}/{} successful", executed_cases - failed_cases, executed_cases);
        println!();
    }
}

// Case Result
//
//

pub struct CaseResult {
    pub case: Case,
    pub result: Result<(), Failure>,
}

impl CaseResult {
    pub fn print(&self) {
        let CaseResult { case, result } = self;
        match result {
            Ok(_) => {
                println!("    - {:40} ✓", case.name)
            }
            Err(err) => {
                println!("    - {:40} ✗", case.name);
                println!();
                println!("    {}", err);
                println!()
            }
        }
    }
}
