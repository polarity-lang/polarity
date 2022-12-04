use super::def::*;
use crate::common::*;

impl HasSpan for Nf {
    fn span(&self) -> Option<codespan::Span> {
        self.info().span
    }
}

impl HasSpan for Neu {
    fn span(&self) -> Option<codespan::Span> {
        self.info().span
    }
}

impl HasSpan for Match {
    fn span(&self) -> Option<codespan::Span> {
        self.info().span
    }
}

impl HasSpan for Comatch {
    fn span(&self) -> Option<codespan::Span> {
        self.info().span
    }
}

impl HasSpan for Case {
    fn span(&self) -> Option<codespan::Span> {
        self.info().span
    }
}

impl HasSpan for Cocase {
    fn span(&self) -> Option<codespan::Span> {
        self.info().span
    }
}
