use std::io;

/// User facing errors
#[derive(Debug)]
pub enum Error {
    IO(io::Error),
    Parser(parser::ParseError<usize, parser::common::OwnedToken, &'static str>),
    Lowering(lowering::LoweringError),
}

pub trait HandleErrorExt<T> {
    fn handle<F: FnOnce(T)>(self, f: F);
}

impl<T> HandleErrorExt<T> for Result<T, Error> {
    fn handle<F: FnOnce(T)>(self, f: F) {
        match self {
            Ok(res) => f(res),
            Err(err) => pretty_print(err),
        }
    }
}

fn pretty_print(err: Error) {
    match err {
        Error::IO(err) => println!("IO error: {}", err),
        Error::Parser(err) => println!("Parse error: {}", err),
        Error::Lowering(err) => println!("Lowering error: {}", err),
    }
}
