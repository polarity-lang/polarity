use std::fmt;

pub enum Dec<Y = ()> {
    Yes(Y),
    No,
}

pub use Dec::*;

impl<Y: fmt::Debug> fmt::Debug for Dec<Y> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Yes(arg) => f.debug_tuple("Yes").field(arg).finish(),
            Self::No => f.debug_tuple("No").finish(),
        }
    }
}
