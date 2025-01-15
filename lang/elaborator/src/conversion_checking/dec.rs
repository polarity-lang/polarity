use std::fmt;

pub enum Dec {
    Yes,
    No,
}

pub use Dec::*;

impl fmt::Debug for Dec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Yes => f.debug_tuple("Yes").finish(),
            Self::No => f.debug_tuple("No").finish(),
        }
    }
}
