use std::error::Error;
use std::fmt;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Arc;

use driver::{Database, FileSource, FileSystemSource, InMemorySource};
use url::Url;

use parser::cst;

use crate::{
    runner::CaseResult,
    suites::{self, Case},
};

pub trait Phase {
    type Out: TestOutput;

    fn new(name: &'static str) -> Self;
    fn name(&self) -> &'static str;
    async fn run(db: &mut Database, uri: &Url) -> Result<Self::Out, driver::Error>;
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
        let database = Database::from_source(source);
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

        // If we are in the phase that is expected to fail, we check
        // whether there is an expected error output.
        let output = config.fail.as_ref().and_then(|fail| {
            if fail == phase.name() {
                self.case.expected()
            } else {
                None
            }
        });

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
                            return Err(PhasesError::ExpectedFailure { got: out2.test_output() });
                        }
                        if let Some(expected) = output {
                            let actual = out2.test_output();
                            if actual != expected {
                                return Err(PhasesError::Mismatch { expected, actual });
                            }
                        }
                        Ok(out2)
                    }
                    Ok(Err(err)) => {
                        let report = tokio::runtime::Runtime::new()
                            .unwrap()
                            .block_on(pretty_error(&mut self.database, &self.case.uri(), err));
                        // There was no panic and `run` returned with an error.
                        self.report_phases
                            .push(PhaseReport { name: phase.name(), output: report.to_string() });
                        if expect_success {
                            return Err(PhasesError::ExpectedSuccess { got: report });
                        }
                        if let Some(expected) = output {
                            let actual = render_report(&report, false);
                            if actual != expected {
                                return Err(PhasesError::Mismatch { expected, actual });
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
            Err(PhasesError::AsExpected { .. }) => Ok(()),
            Err(PhasesError::Mismatch { expected, actual }) => {
                Err(Failure::Mismatch { expected, actual })
            }
            Err(PhasesError::ExpectedFailure { got }) => Err(Failure::ExpectedFailure { got }),
            Err(PhasesError::ExpectedSuccess { got }) => Err(Failure::ExpectedSuccess { got }),
            Err(PhasesError::Panic { msg }) => Err(Failure::Panic { msg }),
        };

        CaseResult { result, case: self.case }
    }
}

#[derive(Debug)]
pub enum Failure {
    Mismatch {
        expected: String,
        actual: String,
    },
    #[allow(clippy::enum_variant_names)]
    ExpectedFailure {
        got: String,
    },
    ExpectedSuccess {
        got: miette::Report,
    },
    Panic {
        msg: String,
    },
}

impl Error for Failure {}

impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Failure::Mismatch { expected, actual } => {
                write!(f, "\n  Expected : {expected}\n  Got      : {actual}")
            }
            Failure::ExpectedFailure { got } => write!(f, "Expected failure, got {got}"),
            Failure::ExpectedSuccess { got } => {
                write!(f, "Expected success, got:\n\n")?;
                let report_str = render_report(got, true);
                write!(f, "{report_str}")
            }
            Failure::Panic { msg } => write!(f, "Code panicked during test execution\n {msg}"),
        }
    }
}

enum PhasesError {
    AsExpected,
    Panic { msg: String },
    Mismatch { expected: String, actual: String },
    ExpectedFailure { got: String },
    ExpectedSuccess { got: miette::Report },
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

    async fn run(db: &mut Database, uri: &Url) -> Result<Self::Out, driver::Error> {
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

    async fn run(db: &mut Database, uri: &Url) -> Result<Self::Out, driver::Error> {
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
    type Out = ast::Module;

    fn new(name: &'static str) -> Self {
        Self { name }
    }

    fn name(&self) -> &'static str {
        self.name
    }

    async fn run(db: &mut Database, uri: &Url) -> Result<Self::Out, driver::Error> {
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
    type Out = Arc<ast::Module>;

    fn new(name: &'static str) -> Self {
        Self { name }
    }

    fn name(&self) -> &'static str {
        self.name
    }

    async fn run(db: &mut Database, uri: &Url) -> Result<Self::Out, driver::Error> {
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

    async fn run(db: &mut Database, uri: &Url) -> Result<Self::Out, driver::Error> {
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

    async fn run(db: &mut Database, uri: &Url) -> Result<Self::Out, driver::Error> {
        // xfunc tests for these examples are currently disabled due to
        // https://github.com/polarity-lang/polarity/issues/317
        if uri.as_str().ends_with("examples/comatches.pol")
            || uri.as_str().ends_with("examples/Webserver.pol")
        {
            return Ok(());
        }

        let type_names = db.all_declared_type_names(uri).await?;

        let new_uri =
            uri.to_string().replacen("file", "inmemory", 1).parse().expect("Failed to parse URI");
        db.source.manage(&new_uri);

        for type_name in type_names.iter().map(|tn| &tn.id) {
            let xfunc_out = db.xfunc(uri, type_name).await?;
            let new_source = db.edited(uri, xfunc_out.edits);
            db.write_source(&new_uri, &new_source.to_string()).await?;
            db.ast(&new_uri).await.map_err(|err| {
                driver::Error::Type(Box::new(elaborator::result::TypeError::Impossible {
                    message: format!("Failed to xfunc {type_name}: {err}"),
                    span: None,
                }))
            })?;
        }

        Ok(())
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

impl TestOutput for ast::Module {
    fn test_output(&self) -> String {
        printer::Print::print_to_string(&self, None)
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

/// Associate error with the relevant source code for pretty-printing.
/// This function differs from `Database::pretty_error` in that it does not display the full URI but only the filename.
/// This is necessary to have reproducible test output (e.g. the `*.expected` files).
async fn pretty_error(db: &mut Database, uri: &Url, err: driver::Error) -> miette::Report {
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

fn render_report(report: &miette::Report, colorize: bool) -> String {
    let theme = if colorize {
        miette::GraphicalTheme::unicode()
    } else {
        miette::GraphicalTheme::unicode_nocolor()
    };
    let handler = miette::GraphicalReportHandler::new_themed(theme);
    let mut output = String::new();
    handler.render_report(&mut output, report.as_ref()).expect("Failed to render report");
    output
}
