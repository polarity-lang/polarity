use std::fmt;

pub enum Dec<Y = (), N = ()> {
    Yes(Y),
    No(N),
}

pub use Dec::*;

impl<Y: fmt::Debug, N: fmt::Debug> fmt::Debug for Dec<Y, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Yes(arg) => f.debug_tuple("Yes").field(arg).finish(),
            Self::No(arg) => f.debug_tuple("No").field(arg).finish(),
        }
    }
}
