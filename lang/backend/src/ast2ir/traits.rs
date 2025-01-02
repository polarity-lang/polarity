use crate::result::BackendError;

pub trait ToIR {
    type Target;

    fn to_ir(&self) -> Result<Self::Target, BackendError>;
}

impl<T: ToIR> ToIR for Vec<T> {
    type Target = Vec<T::Target>;

    fn to_ir(&self) -> Result<Self::Target, BackendError> {
        self.iter().map(|x| x.to_ir()).collect()
    }
}

impl<T: ToIR> ToIR for Option<T> {
    type Target = Option<T::Target>;

    fn to_ir(&self) -> Result<Self::Target, BackendError> {
        match self {
            Some(x) => Ok(Some(x.to_ir()?)),
            None => Ok(None),
        }
    }
}
