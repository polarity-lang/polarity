#[derive(Debug)]
pub enum Error {
    Parser(parser::ParseError<usize, parser::common::OwnedToken, &'static str>),
    Lowering(lowering::LoweringError),
    Type(core::TypeError),
}

pub trait HandleErrorExt<T>: Sized {
    fn handle(self) {
        self.handle_with(|_| ())
    }
    fn handle_with<F: FnOnce(T)>(self, f: F);
}

impl<T> HandleErrorExt<T> for Result<T, Error> {
    fn handle_with<F: FnOnce(T)>(self, f: F) {
        match self {
            Ok(res) => f(res),
            Err(err) => pretty_print(err),
        }
    }
}

fn pretty_print(err: Error) {
    match err {
        Error::Parser(err) => println!("Parse error: {}", err),
        Error::Lowering(err) => println!("Lowering error: {}", err),
        Error::Type(err) => println!("Type error: {}", err),
    }
}
