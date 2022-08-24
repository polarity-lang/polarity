use std::fmt;

pub use lalrpop_util::{lexer::Token, ParseError};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct OwnedToken(pub usize, String);

impl fmt::Display for OwnedToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.1.fmt(f)
    }
}

impl From<Token<'_>> for OwnedToken {
    fn from(token: Token) -> Self {
        OwnedToken(token.0, token.1.to_owned())
    }
}
