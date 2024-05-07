use std::error::Error;
use std::fmt;

use parser::cst;
use printer::PrintToString;
use renaming::Rename;
use syntax::generic::{ForgetTST, Module};

use super::infallible::NoError;

pub trait Phase {
    type In;
    type Out: TestOutput;
    type Err;

    fn new(name: &'static str) -> Self;
    fn name(&self) -> &'static str;
    fn run(input: Self::In) -> Result<Self::Out, Self::Err>;
}

pub struct Phases<O> {
    result: Result<O, PhasesError>,
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

pub trait TestOutput {
    fn test_output(&self) -> String;
}

impl<O> Phases<O>
where
    O: TestOutput,
{
    pub fn start(input: O) -> Phases<O> {
        Phases { result: Ok(input), report_phases: vec![] }
    }

    pub fn then<O2, E, P>(mut self, expect: Expect<P>) -> Phases<O2>
    where
        O2: TestOutput,
        E: Error + 'static,
        P: Phase<In = O, Out = O2, Err = E>,
    {
        let result = self.result.and_then(|out| match P::run(out) {
            Ok(out2) => {
                self.report_phases
                    .push(PhaseReport { name: expect.phase.name(), output: out2.test_output() });
                if !expect.success {
                    return Err(PhasesError::ExpectedFailure { got: out2.test_output() });
                }
                if let Some(expected) = expect.output {
                    let actual = out2.test_output();
                    if actual != expected {
                        return Err(PhasesError::Mismatch { expected, actual });
                    }
                }
                Ok(out2)
            }
            Err(err) => {
                self.report_phases
                    .push(PhaseReport { name: expect.phase.name(), output: err.to_string() });
                if expect.success {
                    return Err(PhasesError::ExpectedSuccess { got: Box::new(err) });
                }
                if let Some(expected) = expect.output {
                    let actual = err.to_string();
                    if actual != expected {
                        return Err(PhasesError::Mismatch { expected, actual });
                    }
                }
                Err(PhasesError::AsExpected { err: Box::new(err) })
            }
        });

        Phases { result, report_phases: self.report_phases }
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

impl Report {
    pub fn print(&self) {
        for PhaseReport { name, output } in &self.phases {
            println!("phase {name}:");
            println!();
            println!("{output}");
            println!();
        }
    }
}

impl TestOutput for String {
    fn test_output(&self) -> String {
        self.to_owned()
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

pub struct Expect<P: Phase> {
    success: bool,
    output: Option<String>,
    phase: P,
}

impl<P: Phase> Expect<P> {
    pub fn new(phase: P, success: bool, output: Option<String>) -> Self {
        Self { success, output, phase }
    }
}

pub struct Parse {
    name: &'static str,
}
pub struct Lower {
    name: &'static str,
}
pub struct Check {
    name: &'static str,
}

pub struct Forget {
    name: &'static str,
}

pub struct Print {
    name: &'static str,
}

impl Phase for Parse {
    type In = String;
    type Out = cst::decls::Module;
    type Err = parser::ParseError;

    fn new(name: &'static str) -> Self {
        Self { name }
    }

    fn name(&self) -> &'static str {
        self.name
    }

    fn run(input: Self::In) -> Result<Self::Out, Self::Err> {
        parser::parse_module(&input)
    }
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

impl Phase for Print {
    type In = Module;
    type Out = String;
    type Err = NoError;

    fn new(name: &'static str) -> Self {
        Self { name }
    }

    fn name(&self) -> &'static str {
        self.name
    }

    fn run(input: Self::In) -> Result<Self::Out, Self::Err> {
        Ok(input.rename().print_to_string(None))
    }
}

impl Phase for Forget {
    type In = Module;
    type Out = Module;
    type Err = NoError;

    fn new(name: &'static str) -> Self {
        Self { name }
    }

    fn name(&self) -> &'static str {
        self.name
    }

    fn run(input: Self::In) -> Result<Self::Out, Self::Err> {
        Ok(input.forget_tst())
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
        self.print_to_string(None)
    }
}
