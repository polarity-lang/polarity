use std::rc::Rc;

use crate::common::*;
use crate::ust;
use codespan::Span;
use derivative::Derivative;
use parser::cst::{HoleKind, Ident};

/// The syntax of normal forms
#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub enum Nf {
    TypCtor {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        info: Option<Span>,
        name: Ident,
        args: Args,
    },
    Ctor {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        info: Option<Span>,
        name: Ident,
        args: Args,
    },
    Type {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        info: Option<Span>,
    },
    Comatch {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        info: Option<Span>,
        name: Label,
        is_lambda_sugar: bool,
        body: Match,
    },
    Neu {
        exp: Neu,
    },
}

impl HasSpan for Nf {
    fn span(&self) -> Option<Span> {
        match self {
            Nf::TypCtor { info, .. } => *info,
            Nf::Ctor { info, .. } => *info,
            Nf::Type { info } => *info,
            Nf::Comatch { info, .. } => *info,
            Nf::Neu { exp } => exp.span(),
        }
    }
}

impl AlphaEq for Nf {
    fn alpha_eq(&self, other: &Self) -> bool {
        self == other
    }
}

/// A term whose normalization is blocked
#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub enum Neu {
    Var {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        info: Option<Span>,
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        name: Ident,
        idx: Idx,
    },
    Dtor {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        info: Option<Span>,
        exp: Rc<Neu>,
        name: Ident,
        args: Args,
    },
    Match {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        info: Option<Span>,
        name: Label,
        on_exp: Rc<Neu>,
        body: Match,
    },
    Hole {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        info: Option<Span>,
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        kind: HoleKind,
    },
}

impl HasSpan for Neu {
    fn span(&self) -> Option<Span> {
        match self {
            Neu::Var { info, .. } => *info,
            Neu::Dtor { info, .. } => *info,
            Neu::Match { info, .. } => *info,
            Neu::Hole { info, .. } => *info,
        }
    }
}

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Match {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub info: Option<Span>,
    pub cases: Vec<Case>,
    pub omit_absurd: bool,
}

impl HasSpan for Match {
    fn span(&self) -> Option<codespan::Span> {
        self.info
    }
}

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Case {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub info: Option<Span>,
    pub name: Ident,
    // TODO: Rename to params
    pub args: ust::TelescopeInst,
    /// Body being `None` represents an absurd pattern
    pub body: Option<Rc<Nf>>,
}

impl HasSpan for Case {
    fn span(&self) -> Option<codespan::Span> {
        self.info
    }
}


#[derive(Debug, Clone)]
pub struct TypApp {
    pub info: Option<Span>,
    pub name: Ident,
    pub args: Args,
}

impl From<TypApp> for Nf {
    fn from(typ_app: TypApp) -> Self {
        let TypApp { info, name, args } = typ_app;

        Nf::TypCtor { info, name, args }
    }
}

pub type Args = Vec<Rc<Nf>>;
