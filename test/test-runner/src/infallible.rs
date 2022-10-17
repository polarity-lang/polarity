use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum NoError {}

impl Error for NoError {}

impl fmt::Display for NoError {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unreachable!()
    }
}
