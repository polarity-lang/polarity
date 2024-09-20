use std::cell::RefCell;
use std::error::Error;
use std::fmt;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Arc;

use query::{Database, DatabaseViewMut, FileSource, FileSystemSource, InMemorySource};
use url::Url;

use parser::cst;

use crate::{
    runner::CaseResult,
    suites::{self, Case},
};

pub trait Phase {
    type Out: TestOutput;
    type Err;

    fn new(name: &'static str) -> Self;
    fn name(&self) -> &'static str;
    fn run(
        view: &mut DatabaseViewMut,
        cst_lookup_table: &mut lowering::LookupTable,
        ast_lookup_table: &mut elaborator::LookupTable,
    ) -> Result<Self::Out, Self::Err>;
}

/// Represents a partially completed run of a testcase, where we have
/// finished running a prefix of all the phases configured for this testcase.
/// The struct is parameterized over the output type of the last phase that
/// has been run.
pub struct PartialRun<O> {
    case: Case,
    database: Database,
    cst_lookup_table: RefCell<lowering::LookupTable>,
    ast_lookup_table: RefCell<elaborator::LookupTable>,
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
        let cst_lookup_table = RefCell::new(lowering::LookupTable::default());
        let ast_lookup_table = RefCell::new(elaborator::LookupTable::default());
        PartialRun {
            case,
            database,
            cst_lookup_table,
            ast_lookup_table,
            result: Ok(()),
            report_phases: vec![],
        }
    }
}

impl<O> PartialRun<O>
where
    O: TestOutput + std::panic::UnwindSafe,
{
    /// Extend this partial run by running one additional phase.
    pub fn then<O2, E, P>(mut self, config: &suites::Config, phase: P) -> PartialRun<O2>
    where
        O2: TestOutput,
        E: Error + 'static,
        P: Phase<Out = O2, Err = E>,
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
        let result = self.result.and_then(|_| {
            // The implementation of the compiler might contain a bug which
            // triggers a panic. We catch this panic here so that we can report the bug as a failing case.
            let view = self.database.open_uri(&self.case.uri()).unwrap();
            let view = RefCell::new(view);
            let cst_lookup_table = &self.cst_lookup_table;
            let ast_lookup_table = &self.ast_lookup_table;

            // Run the phase and catch any panics that might occur.
            // We need to use `AssertUnwindSafe` because the compiler can not automatically
            // guarantee that passing RefCell's across a catch_unwind boundary is safe.
            let run_result = catch_unwind(AssertUnwindSafe(|| {
                let mut view = view.borrow_mut();
                let mut cst_lookup_table = cst_lookup_table.borrow_mut();
                let mut ast_lookup_table = ast_lookup_table.borrow_mut();
                P::run(&mut view, &mut cst_lookup_table, &mut ast_lookup_table)
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
                    // There was no panic and `run` returned with an error.
                    self.report_phases
                        .push(PhaseReport { name: phase.name(), output: err.to_string() });
                    if expect_success {
                        return Err(PhasesError::ExpectedSuccess { got: Box::new(err) });
                    }
                    if let Some(expected) = output {
                        let actual = err.to_string();
                        if actual != expected {
                            return Err(PhasesError::Mismatch { expected, actual });
                        }
                    }
                    Err(PhasesError::AsExpected)
                }
                Err(_) => {
                    // There was a panic
                    self.report_phases.push(PhaseReport {
                        name: phase.name(),
                        output: "Panic occurred".to_string(),
                    });
                    Err(PhasesError::Panic)
                }
            }
        });

        PartialRun {
            database: self.database,
            case: self.case,
            cst_lookup_table: self.cst_lookup_table,
            ast_lookup_table: self.ast_lookup_table,
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
            Err(PhasesError::Panic) => Err(Failure::Panic),
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
        got: Box<dyn Error>,
    },
    Panic,
}

impl Error for Failure {}

impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Failure::Mismatch { expected, actual } => {
                write!(f, "\n  Expected : {expected}\n  Got      : {actual}")
            }
            Failure::ExpectedFailure { got } => write!(f, "Expected failure, got {got}"),
            Failure::ExpectedSuccess { got } => write!(f, "Expected success, got {got}"),
            Failure::Panic => write!(f, "Code panicked during test execution"),
        }
    }
}

enum PhasesError {
    AsExpected,
    Panic,
    Mismatch { expected: String, actual: String },
    ExpectedFailure { got: String },
    ExpectedSuccess { got: Box<dyn Error> },
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
    type Err = query::Error;

    fn new(name: &'static str) -> Self {
        Self { name }
    }

    fn name(&self) -> &'static str {
        self.name
    }

    fn run(
        view: &mut DatabaseViewMut,
        _: &mut lowering::LookupTable,
        _: &mut elaborator::LookupTable,
    ) -> Result<Self::Out, Self::Err> {
        view.load_cst(&view.uri.clone())
    }
}

pub struct Imports {
    name: &'static str,
}

impl Phase for Imports {
    type Out = ();
    type Err = query::Error;

    fn new(name: &'static str) -> Self {
        Self { name }
    }

    fn name(&self) -> &'static str {
        self.name
    }

    fn run(
        view: &mut DatabaseViewMut,
        cst_lookup_table: &mut lowering::LookupTable,
        ast_lookup_table: &mut elaborator::LookupTable,
    ) -> Result<Self::Out, Self::Err> {
        view.load_imports(cst_lookup_table, ast_lookup_table)
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
    type Err = query::Error;

    fn new(name: &'static str) -> Self {
        Self { name }
    }

    fn name(&self) -> &'static str {
        self.name
    }

    fn run(
        view: &mut DatabaseViewMut,
        cst_lookup_table: &mut lowering::LookupTable,
        _: &mut elaborator::LookupTable,
    ) -> Result<Self::Out, Self::Err> {
        view.load_ust(&view.uri.clone(), cst_lookup_table)
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
    type Err = query::Error;

    fn new(name: &'static str) -> Self {
        Self { name }
    }

    fn name(&self) -> &'static str {
        self.name
    }

    fn run(
        view: &mut DatabaseViewMut,
        cst_lookup_table: &mut lowering::LookupTable,
        ast_lookup_table: &mut elaborator::LookupTable,
    ) -> Result<Self::Out, Self::Err> {
        view.load_ast(&view.uri.clone(), cst_lookup_table, ast_lookup_table)
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
    type Err = query::Error;

    fn new(name: &'static str) -> Self {
        Self { name }
    }

    fn name(&self) -> &'static str {
        self.name
    }

    fn run(
        view: &mut DatabaseViewMut,
        cst_lookup_table: &mut lowering::LookupTable,
        ast_lookup_table: &mut elaborator::LookupTable,
    ) -> Result<Self::Out, Self::Err> {
        let output = view.print_to_string(&view.uri.clone())?;
        view.write_source(&view.uri.clone(), &output)?;
        *cst_lookup_table = Default::default();
        *ast_lookup_table = Default::default();
        Ok(output)
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
