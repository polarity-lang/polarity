use std::error::Error;
use std::fmt;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;

use polarity_lang_driver::{
    AppResult, Database, FileSource, FileSystemSource, InMemorySource, render_reports_to_string,
};
use polarity_lang_printer::Print as _;
use url::Url;

use polarity_lang_parser::cst;

use crate::{
    runner::CaseResult,
    suites::{self, Case},
};

pub trait Phase {
    type Out: TestOutput;

    fn new(name: &'static str) -> Self;
    fn name(&self) -> &'static str;
    async fn run(db: &mut Database, uri: &Url) -> AppResult<Self::Out>;
}

/// Represents a partially completed run of a testcase, where we have
/// finished running a prefix of all the phases configured for this testcase.
/// The struct is parameterized over the output type of the last phase that
/// has been run.
pub struct PartialRun<O> {
    case: Case,
    database: Database,
    /// The result of the last run phase.
    result: Result<O, PhasesError>,
    /// A textual report about all the previously run phases.
    report_phases: Vec<PhaseReport>,
}

#[allow(dead_code)]
pub struct PhaseReport {
    pub name: &'static str,
    pub output: String,
}

impl PartialRun<()> {
    /// Start a new partial run for a testcase with the initial input.
    pub fn start(case: Case) -> PartialRun<()> {
        let mut source = InMemorySource::new();
        source.insert(case.uri(), case.content().unwrap());
        let source = source.fallback_to(FileSystemSource::new(&case.path));
        let database = Database::new(source);
        PartialRun { case, database, result: Ok(()), report_phases: vec![] }
    }
}

impl<O> PartialRun<O>
where
    O: TestOutput + std::panic::UnwindSafe,
{
    /// Extend this partial run by running one additional phase.
    pub fn then<O2, P>(mut self, config: &suites::Config, phase: P) -> PartialRun<O2>
    where
        O2: TestOutput,
        P: Phase<Out = O2>,
    {
        // Whether we expect a success in this phase.
        let expect_success = config.fail.as_ref().map(|fail| fail != phase.name()).unwrap_or(true);

        let expected_output = match config.fail.as_deref() {
            Some(fail) if fail == phase.name() => self.case.expected(phase.name()),
            None => self.case.expected(phase.name()),
            _ => None,
        };

        // Run the phase and handle the result
        let result =
            self.result.and_then(|_| {
                // The implementation of the compiler might contain a bug which
                // triggers a panic. We catch this panic here so that we can report the bug as a failing case.

                // Run the phase and catch any panics that might occur.
                // We need to use `AssertUnwindSafe` because the compiler can not automatically
                // guarantee that passing mutable references across a catch_unwind boundary is safe.
                let run_result = catch_unwind(AssertUnwindSafe(|| {
                    tokio::runtime::Runtime::new()
                        .unwrap()
                        .block_on(P::run(&mut self.database, &self.case.uri()))
                }));

                match run_result {
                    Ok(Ok(out2)) => {
                        // There was no panic and `run` returned with a result.
                        self.report_phases
                            .push(PhaseReport { name: phase.name(), output: out2.test_output() });
                        if !expect_success {
                            return Err(PhasesError::ExpectedFailure {
                                phase: phase.name(),
                                got: out2.test_output(),
                            });
                        }
                        if let Some(expected) = expected_output {
                            let actual = out2.test_output();
                            if actual != expected {
                                return Err(PhasesError::Mismatch {
                                    phase: phase.name(),
                                    expected,
                                    actual,
                                });
                            }
                        }
                        Ok(out2)
                    }
                    Ok(Err(err)) => {
                        let reports = tokio::runtime::Runtime::new()
                            .unwrap()
                            .block_on(pretty_errors(&mut self.database, &self.case.uri(), err));

                        // There was no panic and `run` returned with an error.
                        self.report_phases.push(PhaseReport {
                            name: phase.name(),
                            output: render_reports_to_string(&reports, true),
                        });
                        if expect_success {
                            return Err(PhasesError::ExpectedSuccess {
                                phase: phase.name(),
                                got: reports,
                            });
                        }
                        if let Some(expected) = expected_output {
                            let actual = render_reports_to_string(&reports, false);
                            if actual != expected {
                                return Err(PhasesError::Mismatch {
                                    phase: phase.name(),
                                    expected,
                                    actual,
                                });
                            }
                        }
                        Err(PhasesError::AsExpected)
                    }
                    Err(err) => {
                        // There was a panic
                        self.report_phases.push(PhaseReport {
                            name: phase.name(),
                            output: "Panic occurred".to_string(),
                        });
                        Err(PhasesError::Panic { msg: err.downcast::<&str>().unwrap().to_string() })
                    }
                }
            });

        PartialRun {
            database: self.database,
            case: self.case,
            result,
            report_phases: self.report_phases,
        }
    }

    pub fn report(self) -> CaseResult {
        let result = match self.result {
            Ok(_) => Ok(()),
            Err(PhasesError::AsExpected) => Ok(()),
            Err(PhasesError::Mismatch { phase, expected, actual }) => {
                Err(Failure::Mismatch { phase, expected, actual })
            }
            Err(PhasesError::ExpectedFailure { phase, got }) => {
                Err(Failure::ExpectedFailure { phase, got })
            }
            Err(PhasesError::ExpectedSuccess { phase, got }) => {
                Err(Failure::ExpectedSuccess { phase, got })
            }
            Err(PhasesError::Panic { msg }) => Err(Failure::Panic { msg }),
        };

        CaseResult { result, case: self.case }
    }
}

/// A test case failed.
#[derive(Debug)]
pub enum Failure {
    /// The output of a phase did not match the expected output.
    Mismatch { phase: &'static str, expected: String, actual: String },
    /// The test was expected to fail, but it succeeded.
    #[allow(clippy::enum_variant_names)]
    ExpectedFailure { phase: &'static str, got: String },
    /// The test was expected to succeed, but it failed.
    ExpectedSuccess { phase: &'static str, got: Vec<miette::Report> },
    /// The test panicked during execution.
    Panic { msg: String },
}

impl Error for Failure {}

impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Failure::Mismatch { phase: _, expected, actual } => {
                write!(f, "\n  Expected : {expected}\n  Got      : {actual}")
            }
            Failure::ExpectedFailure { phase, got } => {
                write!(f, "Expected failure in phase {phase}, but got {got}")
            }
            Failure::ExpectedSuccess { phase, got } => {
                write!(f, "Expected success in phase {phase}, but got:\n\n")?;
                let report_str = render_reports_to_string(got, true);
                write!(f, "{report_str}")
            }
            Failure::Panic { msg } => write!(f, "Code panicked during test execution\n {msg}"),
        }
    }
}

enum PhasesError {
    AsExpected,
    Panic { msg: String },
    Mismatch { phase: &'static str, expected: String, actual: String },
    ExpectedFailure { phase: &'static str, got: String },
    ExpectedSuccess { phase: &'static str, got: Vec<miette::Report> },
}

// Parse Phase
//
// This phase transforms a string into a cst (concrete syntax tree).
// We use this phase to test the implementation of the lexer and parser.
pub struct Parse {
    name: &'static str,
}

impl Phase for Parse {
    type Out = Arc<cst::decls::Module>;

    fn new(name: &'static str) -> Self {
        Self { name }
    }

    fn name(&self) -> &'static str {
        self.name
    }

    async fn run(db: &mut Database, uri: &Url) -> AppResult<Self::Out> {
        db.cst(uri).await
    }
}

pub struct Imports {
    name: &'static str,
}

impl Phase for Imports {
    type Out = ();

    fn new(name: &'static str) -> Self {
        Self { name }
    }

    fn name(&self) -> &'static str {
        self.name
    }

    async fn run(db: &mut Database, uri: &Url) -> AppResult<Self::Out> {
        db.load_imports(uri).await
    }
}

/// Lower Phase
///
/// This phase lowers a module from its cst (concrete syntax tree) representation
/// to its ast (abstract syntax tree) representation. We use this phase to test
/// the implementation of lowering.
pub struct Lower {
    name: &'static str,
}

impl Phase for Lower {
    type Out = polarity_lang_ast::Module;

    fn new(name: &'static str) -> Self {
        Self { name }
    }

    fn name(&self) -> &'static str {
        self.name
    }

    async fn run(db: &mut Database, uri: &Url) -> AppResult<Self::Out> {
        db.ust(uri).await.map(|x| (*x).clone())
    }
}

// Check Phase
//
// This phase elaborates a module which has not been typechecked and
// generates a type-annotated module. We use this phase to test the
// elaborator

pub struct Check {
    name: &'static str,
}

impl Phase for Check {
    type Out = Arc<polarity_lang_ast::Module>;

    fn new(name: &'static str) -> Self {
        Self { name }
    }

    fn name(&self) -> &'static str {
        self.name
    }

    async fn run(db: &mut Database, uri: &Url) -> AppResult<Self::Out> {
        db.ast(uri).await
    }
}

// Print Phase
//
// This phase prettyprints a module to a string and cannot fail.
// We use this phase to test whether the output of the print phase
// can be parsed again by the parser. This ensures that the prettyprinter
// is implemented correctly.

pub struct Print {
    name: &'static str,
}

impl Phase for Print {
    type Out = String;

    fn new(name: &'static str) -> Self {
        Self { name }
    }

    fn name(&self) -> &'static str {
        self.name
    }

    async fn run(db: &mut Database, uri: &Url) -> AppResult<Self::Out> {
        let output = db.print_to_string(uri).await?;
        db.write_source(uri, &output).await?;
        Ok(output)
    }
}

// Xfunc Phase
//
// This phase runs xfunctionalization on each type in the module, and tests
// whether the resulting output still typechecks.

pub struct Xfunc {
    name: &'static str,
}

impl Phase for Xfunc {
    type Out = ();

    fn new(name: &'static str) -> Self {
        Self { name }
    }

    fn name(&self) -> &'static str {
        self.name
    }

    async fn run(db: &mut Database, uri: &Url) -> AppResult<Self::Out> {
        // xfunc tests for these examples are currently disabled due to
        // https://github.com/polarity-lang/polarity/issues/317
        if uri.as_str().ends_with("suites/success/023-comatches.pol")
            || uri.as_str().ends_with("suites/success/036-webserver.pol")
        {
            return Ok(());
        }

        let type_names = db.all_declared_type_names(uri).await?;

        let new_uri =
            uri.to_string().replacen("file", "inmemory", 1).parse().expect("Failed to parse URI");
        db.source.register(&new_uri);

        for type_name in type_names.iter().map(|tn| &tn.id) {
            let xfunc_out = db.xfunc(uri, type_name).await?;
            let new_source = db.edited(uri, xfunc_out.edits);
            db.write_source(&new_uri, &new_source.to_string()).await?;
            db.ast(&new_uri).await.map_err(|err| {
                polarity_lang_driver::AppError::Type(Box::new(
                    polarity_lang_elaborator::result::TypeError::Impossible {
                        message: format!("Failed to xfunc {type_name}: {err:?}"),
                        span: None,
                    },
                ))
            })?;
        }

        Ok(())
    }
}

// IR Phase
//
// This phase generates the intermediate representation of the module.

pub struct IR {
    name: &'static str,
}

impl Phase for IR {
    type Out = String;

    fn new(name: &'static str) -> Self {
        Self { name }
    }

    fn name(&self) -> &'static str {
        self.name
    }

    async fn run(db: &mut Database, uri: &Url) -> AppResult<Self::Out> {
        let ir = db.ir(uri).await?;
        let pretty_ir = ir.print_to_string(None);
        Ok(pretty_ir)
    }
}

// TestOutput

pub trait TestOutput {
    fn test_output(&self) -> String;
}

impl TestOutput for () {
    fn test_output(&self) -> String {
        "".to_owned()
    }
}

impl TestOutput for String {
    fn test_output(&self) -> String {
        self.to_owned()
    }
}

impl TestOutput for cst::decls::Module {
    fn test_output(&self) -> String {
        // TODO: Improve test output
        format!("{self:?}")
    }
}

impl TestOutput for polarity_lang_ast::Module {
    fn test_output(&self) -> String {
        polarity_lang_printer::Print::print_to_string(&self, None)
    }
}

impl TestOutput for Url {
    fn test_output(&self) -> String {
        self.to_string()
    }
}

impl<T: TestOutput> TestOutput for Arc<T> {
    fn test_output(&self) -> String {
        self.as_ref().test_output()
    }
}

impl<S: TestOutput, T: TestOutput> TestOutput for (S, T) {
    fn test_output(&self) -> String {
        let (x, y) = self;
        format!("({},{})", x.test_output(), y.test_output())
    }
}

async fn pretty_errors(
    db: &mut Database,
    uri: &Url,
    errs: polarity_lang_driver::AppErrors,
) -> Vec<miette::Report> {
    let errs = errs.into_errors();
    let mut reports = Vec::with_capacity(errs.len());
    for err in errs {
        reports.push(pretty_error(db, uri, err).await);
    }
    reports
}

/// Associate error with the relevant source code for pretty-printing.
/// This function differs from `Database::pretty_error` in that it does not display the full URI but only the filename.
/// This is necessary to have reproducible test output (e.g. the `*.expected` files).
async fn pretty_error(
    db: &mut Database,
    uri: &Url,
    err: polarity_lang_driver::AppError,
) -> miette::Report {
    let miette_error: miette::Error = err.into();
    let source = db.source(uri).await.expect("Failed to get source");
    let filepath = uri.to_file_path().expect("Failed to convert URI to file path");

    let filename = filepath
        .file_name()
        .expect("Failed to get file name")
        .to_str()
        .expect("Failed to convert file name to string");
    miette_error.with_source_code(miette::NamedSource::new(filename, source.to_owned()))
}
