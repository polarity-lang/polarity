pub trait Extract {
    type Target;

    fn extract(self) -> Self::Target;
}

impl<T> Extract for Result<T, T> {
    type Target = T;

    fn extract(self) -> Self::Target {
        match self {
            Ok(x) => x,
            Err(x) => x,
        }
    }
}
