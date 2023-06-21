use std::rc::Rc;

use derivative::Derivative;

use crate::common::*;
use crate::cst::{HoleKind, Ident};
use crate::ust;

/// The syntax of normal forms
#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub enum Nf {
    TypCtor {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        info: Info,
        name: Ident,
        args: Args,
    },
    Ctor {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        info: Info,
        name: Ident,
        args: Args,
    },
    Type {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        info: Info,
    },
    Comatch {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        info: Info,
        name: Label,
        is_lambda_sugar: bool,
        body: Comatch,
    },
    Neu {
        exp: Neu,
    },
}

/// A term whose normalization is blocked
#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub enum Neu {
    Var {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        info: Info,
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        name: Ident,
        idx: Idx,
    },
    Dtor {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        info: ust::Info,
        exp: Rc<Neu>,
        name: Ident,
        args: Args,
    },
    Match {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        info: Info,
        name: Label,
        on_exp: Rc<Neu>,
        body: Match,
    },
    Hole {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        info: Info,
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        kind: HoleKind,
    },
}

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Match {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub info: Info,
    pub cases: Vec<Case>,
    pub omit_absurd: bool,
}

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Comatch {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub info: Info,
    // TODO: Consider renaming this field to "cocases"
    pub cases: Vec<Cocase>,
    pub omit_absurd: bool,
}

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Case {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub info: Info,
    pub name: Ident,
    // TODO: Rename to params
    pub args: ust::TelescopeInst,
    /// Body being `None` represents an absurd pattern
    pub body: Option<Rc<Nf>>,
}

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Cocase {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub info: Info,
    pub name: Ident,
    // TODO: Rename to params
    pub args: ust::TelescopeInst,
    /// Body being `None` represents an absurd pattern
    pub body: Option<Rc<Nf>>,
}

#[derive(Debug, Clone)]
pub struct TypApp {
    pub info: Info,
    pub name: Ident,
    pub args: Args,
}

impl From<TypApp> for Nf {
    fn from(typ_app: TypApp) -> Self {
        let TypApp { info, name, args } = typ_app;

        Nf::TypCtor { info, name, args }
    }
}

pub type Info = ust::Info;
pub type Args = Vec<Rc<Nf>>;
