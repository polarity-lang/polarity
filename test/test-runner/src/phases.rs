use std::error::Error;
use std::fmt;

use printer::PrintToString;
use syntax::{ast, cst, elab};

pub trait Phase {
    type In;
    type Out: TestOutput;
    type Err;

    fn name() -> &'static str;
    fn run(input: &Self::In) -> Result<Self::Out, Self::Err>;
}

pub struct Phases<O> {
    result: Result<O, PhasesError>,
}

pub trait TestOutput {
    fn test_output(&self) -> String;
}

impl<O> Phases<O>
where
    O: TestOutput,
{
    pub fn start(input: O) -> Phases<O> {
        Phases { result: Ok(input) }
    }

    pub fn then<O2, E, P>(self, expect: Expect<P>) -> Phases<O2>
    where
        O2: TestOutput,
        E: Error + 'static,
        P: Phase<In = O, Out = O2, Err = E>,
    {
        let result = self.result.and_then(|out| match P::run(&out) {
            Ok(out2) => {
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

        Phases { result }
    }

    pub fn end(self) -> Result<String, Failure> {
        match self.result {
            Ok(out) => Ok(out.test_output()),
            Err(PhasesError::AsExpected { err }) => Ok(format!("{:?}", err)),
            Err(PhasesError::Mismatch { expected, actual }) => {
                Err(Failure::Mismatch { expected, actual })
            }
            Err(PhasesError::ExpectedFailure { got }) => Err(Failure::ExpectedFailure { got }),
            Err(PhasesError::ExpectedSuccess { got }) => Err(Failure::ExpectedSuccess { got }),
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
    Mismatch { expected: String, actual: String },
    ExpectedFailure { got: String },
    ExpectedSuccess { got: Box<dyn Error> },
}

impl Error for Failure {}

impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Failure::Mismatch { expected, actual } => {
                write!(f, "Expected {}, got {}", expected, actual)
            }
            Failure::ExpectedFailure { got } => write!(f, "Expected failure, got {}", got),
            Failure::ExpectedSuccess { got } => write!(f, "Expected success, got {}", got),
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
    _phase: P,
}

impl<P: Phase> Expect<P> {
    pub fn new(phase: P, success: bool, output: Option<String>) -> Self {
        Self { success, output, _phase: phase }
    }
}

pub struct Parse;
pub struct Lower;
pub struct Check;

impl Phase for Parse {
    type In = String;
    type Out = cst::Prg;
    type Err = parser::ParseError<usize, parser::common::OwnedToken, &'static str>;

    fn name() -> &'static str {
        "parse"
    }

    fn run(input: &Self::In) -> Result<Self::Out, Self::Err> {
        parser::cst::parse_program(input)
    }
}

impl Phase for Lower {
    type In = cst::Prg;
    type Out = ast::Prg;
    type Err = lowering::LoweringError;

    fn name() -> &'static str {
        "lower"
    }

    fn run(input: &Self::In) -> Result<Self::Out, Self::Err> {
        lowering::lower(input)
    }
}

impl Phase for Check {
    type In = ast::Prg;
    type Out = elab::Prg;
    type Err = core::TypeError;

    fn name() -> &'static str {
        "check"
    }

    fn run(input: &Self::In) -> Result<Self::Out, Self::Err> {
        core::check(input)
    }
}

impl TestOutput for cst::Prg {
    fn test_output(&self) -> String {
        // TODO: Improve test output
        format!("{:?}", self)
    }
}

impl TestOutput for ast::Prg {
    fn test_output(&self) -> String {
        self.print_to_string()
    }
}

impl TestOutput for elab::Prg {
    fn test_output(&self) -> String {
        // TODO: Improve test output
        format!("{:?}", self)
    }
}
