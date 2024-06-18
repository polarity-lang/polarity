use std::error::Error;
use std::fmt;

use url::Url;

use parser::cst;

use renaming::Rename;
use syntax::ast::Module;

use crate::suites::{self, Case};

pub trait Phase {
    type In;
    type Out: TestOutput;
    type Err;

    fn new(name: &'static str) -> Self;
    fn name(&self) -> &'static str;
    fn run(input: Self::In) -> Result<Self::Out, Self::Err>;
}

/// Represents a partially completed run of a testcase, where we have
/// finished running a prefix of all the phases configured for this testcase.
/// The struct is parameterized over the output type of the last phase that
/// has been run.
pub struct PartialRun<O> {
    /// The result of the last run.
    result: Result<O, PhasesError>,
    /// A textual report about all the previously run phases.
    report_phases: Vec<PhaseReport>,
}

pub struct Report {
    pub phases: Vec<PhaseReport>,
    pub result: Result<String, Failure>,
}

pub struct PhaseReport {
    pub name: &'static str,
    pub output: String,
}

impl<O> PartialRun<O>
where
    O: TestOutput,
{
    /// Start a new partial run for a testcase with the initial input.
    pub fn start(input: O) -> PartialRun<O> {
        PartialRun { result: Ok(input), report_phases: vec![] }
    }

    /// Extend this partial run by running one additional phase.
    pub fn then<O2, E, P>(
        mut self,
        config: &suites::Config,
        case: &Case,
        phase: P,
    ) -> PartialRun<O2>
    where
        O2: TestOutput,
        E: Error + 'static,
        P: Phase<In = O, Out = O2, Err = E>,
    {
        // Whether we expect a success in this phase.
        let success = config.fail.as_ref().map(|fail| fail != phase.name()).unwrap_or(true);

        // If we are in the phase that is expected to fail, we check
        // whether there is an expected error output.
        let output = config.fail.as_ref().and_then(|fail| {
            if fail == phase.name() {
                case.expected()
            } else {
                None
            }
        });

        // Run the phase and handle the result
        let result = self.result.and_then(|out| match P::run(out) {
            Ok(out2) => {
                self.report_phases
                    .push(PhaseReport { name: phase.name(), output: out2.test_output() });
                if !success {
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
            Err(err) => {
                self.report_phases
                    .push(PhaseReport { name: phase.name(), output: err.to_string() });
                if success {
                    return Err(PhasesError::ExpectedSuccess { got: Box::new(err) });
                }
                if let Some(expected) = output {
                    let actual = err.to_string();
                    if actual != expected {
                        return Err(PhasesError::Mismatch { expected, actual });
                    }
                }
                Err(PhasesError::AsExpected { err: Box::new(err) })
            }
        });

        PartialRun { result, report_phases: self.report_phases }
    }

    pub fn report(self) -> Report {
        let result = match self.result {
            Ok(out) => Ok(out.test_output()),
            Err(PhasesError::AsExpected { err }) => Ok(format!("{err:?}")),
            Err(PhasesError::Mismatch { expected, actual }) => {
                Err(Failure::Mismatch { expected, actual })
            }
            Err(PhasesError::ExpectedFailure { got }) => Err(Failure::ExpectedFailure { got }),
            Err(PhasesError::ExpectedSuccess { got }) => Err(Failure::ExpectedSuccess { got }),
        };

        Report { result, phases: self.report_phases }
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
        }
    }
}

enum PhasesError {
    AsExpected { err: Box<dyn Error> },
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
    type In = (Url, String);
    type Out = cst::decls::Module;
    type Err = parser::ParseError;

    fn new(name: &'static str) -> Self {
        Self { name }
    }

    fn name(&self) -> &'static str {
        self.name
    }

    fn run(input: Self::In) -> Result<Self::Out, Self::Err> {
        let (uri, input) = &input;
        parser::parse_module(uri.clone(), input)
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
    type In = cst::decls::Module;
    type Out = Module;
    type Err = lowering::LoweringError;

    fn new(name: &'static str) -> Self {
        Self { name }
    }

    fn name(&self) -> &'static str {
        self.name
    }

    fn run(input: Self::In) -> Result<Self::Out, Self::Err> {
        lowering::lower_module(&input)
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
    type In = Module;
    type Out = Module;
    type Err = elaborator::result::TypeError;

    fn new(name: &'static str) -> Self {
        Self { name }
    }

    fn name(&self) -> &'static str {
        self.name
    }

    fn run(input: Self::In) -> Result<Self::Out, Self::Err> {
        elaborator::typechecker::check(&input)
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

#[derive(Debug)]
pub enum NoError {}

impl Error for NoError {}

impl fmt::Display for NoError {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unreachable!()
    }
}

impl Phase for Print {
    type In = Module;
    type Out = (Url, String);
    type Err = NoError;

    fn new(name: &'static str) -> Self {
        Self { name }
    }

    fn name(&self) -> &'static str {
        self.name
    }

    fn run(input: Self::In) -> Result<Self::Out, Self::Err> {
        Ok((input.uri.clone(), printer::Print::print_to_string(&input.rename(), None)))
    }
}

// TestOutput
//
//

pub trait TestOutput {
    fn test_output(&self) -> String;
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

impl TestOutput for Module {
    fn test_output(&self) -> String {
        printer::Print::print_to_string(&self, None)
    }
}

impl TestOutput for Url {
    fn test_output(&self) -> String {
        self.to_string()
    }
}

impl<S: TestOutput, T: TestOutput> TestOutput for (S, T) {
    fn test_output(&self) -> String {
        let (x, y) = self;
        format!("({},{})", x.test_output(), y.test_output())
    }
}
