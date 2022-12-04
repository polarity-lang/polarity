use super::def::*;
use crate::common::*;

impl HasInfo for Nf {
    type Info = Info;

    fn info(&self) -> Self::Info {
        match self {
            Nf::TypCtor { info, .. } => info.clone(),
            Nf::Ctor { info, .. } => info.clone(),
            Nf::Type { info } => info.clone(),
            Nf::Comatch { info, .. } => info.clone(),
            Nf::Neu { exp } => exp.info(),
        }
    }
}

impl HasInfo for Neu {
    type Info = Info;

    fn info(&self) -> Self::Info {
        match self {
            Neu::Var { info, .. } => info.clone(),
            Neu::Dtor { info, .. } => info.clone(),
            Neu::Match { info, .. } => info.clone(),
        }
    }
}

impl HasInfo for Match {
    type Info = Info;

    fn info(&self) -> Self::Info {
        self.info.clone()
    }
}

impl HasInfo for Comatch {
    type Info = Info;

    fn info(&self) -> Self::Info {
        self.info.clone()
    }
}

impl HasInfo for Case {
    type Info = Info;

    fn info(&self) -> Self::Info {
        self.info.clone()
    }
}

impl HasInfo for Cocase {
    type Info = Info;

    fn info(&self) -> Self::Info {
        self.info.clone()
    }
}
