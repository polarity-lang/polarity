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

impl<Y, N> Dec<Y, N> {
    pub fn map_yes<F, Y2>(self, f: F) -> Dec<Y2, N>
    where
        F: FnOnce(Y) -> Y2,
    {
        match self {
            Yes(arg) => Yes(f(arg)),
            No(arg) => No(arg),
        }
    }

    pub fn map_no<F, N2>(self, f: F) -> Dec<Y, N2>
    where
        F: FnOnce(N) -> N2,
    {
        match self {
            Yes(arg) => Yes(arg),
            No(arg) => No(f(arg)),
        }
    }

    pub fn ok_yes(self) -> Result<Y, N> {
        match self {
            Yes(arg) => Ok(arg),
            No(arg) => Err(arg),
        }
    }

    pub fn ok_no(self) -> Result<N, Y> {
        match self {
            Yes(arg) => Err(arg),
            No(arg) => Ok(arg),
        }
    }
}
