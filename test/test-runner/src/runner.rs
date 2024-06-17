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

    pub fn run(&self, args: &Args) -> RunResult {
        let search_string = match &args.filter {
            None => ALL_GLOB,
            Some(str) => &str,
        };
        let mut results: Vec<_> = self.index.searcher().search(search_string).collect();
        results.sort_by(|a, b| a.suite.cmp(&b.suite).then(a.name.cmp(&b.name)));

        let mut failure_count = 0;
        let cases_count = results.len();

        if results.is_empty() {
            return RunResult { results: vec![], failure_count: 0, cases_count: 0 };
        }

        let mut suite_results = Vec::new();
        let mut case_results = Vec::new();
        let mut curr_suite = self.suites[&results.first().unwrap().suite].clone();
        let mut curr_config = curr_suite.config.clone();

        for case in results {
            if case.suite != curr_suite.name {
                suite_results.push(SuiteResult { suite: curr_suite, results: case_results });
                curr_suite = self.suites[&case.suite].clone();
                case_results = Vec::new();
                curr_config = curr_suite.config.clone();
            }
            let report = self.run_case(&curr_config, &case);
            if args.debug {
                report.print();
            }
            let result = report.result;
            if result.is_err() {
                failure_count += 1;
            }
            case_results.push(CaseResult { result, case })
        }

        suite_results.push(SuiteResult { suite: curr_suite, results: case_results });

        RunResult { results: suite_results, cases_count, failure_count }
    }

    pub fn run_suite(&self, args: &Args, suite: &suites::Suite) -> SuiteResult {
        // We first have to filter out those cases which should not be run.
        let search_string = match &args.filter {
            None => ALL_GLOB,
            Some(str) => &str,
        };
        let matching_cases: Vec<Case> = self.index.searcher().search(search_string).collect();

        let mut results: Vec<CaseResult> = vec![];

        for case in &suite.cases {
            if !matching_cases.contains(&case) {
                continue;
            }

            let report = self.run_case(&suite.config, case);

            if args.debug {
                report.print();
            }

            let result = CaseResult { case: case.clone(), result: report.result };
            results.push(result);
        }
        SuiteResult { suite: suite.clone(), results }
    }

    pub fn run_case(&self, config: &suites::Config, case: &Case) -> Report {
        let canonicalized_path = case.path.clone().canonicalize().unwrap();
        let uri = Url::from_file_path(canonicalized_path).unwrap();
        let input = (uri, case.content().unwrap());

        PartialRun::start(input)
            .then(config, case, Parse::new("parse"))
            .then(config, case, Lower::new("lower"))
            .then(config, case, Check::new("check"))
            .then(config, case, Print::new("print"))
            .then(config, case, Parse::new("reparse"))
            .then(config, case, Lower::new("relower"))
            .then(config, case, Check::new("recheck"))
            .report()
    }
}

pub struct RunResult {
    results: Vec<SuiteResult>,
    failure_count: usize,
    cases_count: usize,
}

pub struct SuiteResult {
    suite: Suite,
    results: Vec<CaseResult>,
}

pub struct CaseResult {
    case: Case,
    result: Result<String, Failure>,
}

impl RunResult {
    pub fn success(&self) -> bool {
        self.failure_count == 0
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
        for SuiteResult { suite, results } in &self.results {
            println!("Suite \"{}\":", suite.name);
            let mut success_count = 0;
            for CaseResult { case, result } in results {
                match result {
                    Ok(_) => success_count += 1,
                    Err(err) => println!("{}: {}", case.name, err),
                }
            }
            println!("{}/{} successful", success_count, results.len());
            println!();
        }
        println!(
            "In total: {}/{} successful",
            self.cases_count - self.failure_count,
            self.cases_count
        );
    }
}
