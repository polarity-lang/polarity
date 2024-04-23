use codespan::Span;
use printer::PrintToString;

use syntax::ctx::values::{Binder as TypeCtxBinder, TypeCtx};

// HoverInfo
//
// Types which contain the information that is displayed in a
// code editor when hovering over a point in the program code.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HoverInfo {
    /// The source code location to which the content applies
    pub span: Span,
    /// The information that should be displayed on hover
    pub content: HoverInfoContent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HoverInfoContent {
    GenericInfo(GenericInfo),
    VariableInfo(VariableInfo),
}

// TODO: Completely remove generic info and replace it with concrete types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenericInfo {
    pub typ: String,
    pub ctx: Option<Ctx>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariableInfo {
    pub typ: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ctx {
    pub bound: Vec<Vec<Binder>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Binder {
    pub name: String,
    pub typ: String,
}

impl From<TypeCtx> for Ctx {
    fn from(ctx: TypeCtx) -> Self {
        let bound =
            ctx.bound.into_iter().map(|tel| tel.into_iter().map(Into::into).collect()).collect();
        Ctx { bound }
    }
}

impl From<TypeCtxBinder> for Binder {
    fn from(binder: TypeCtxBinder) -> Self {
        Binder { name: binder.name, typ: binder.typ.print_to_string(None) }
    }
}

// Item
//
//

#[derive(PartialEq, Eq, Clone)]
pub enum Item {
    Data(String),
    Codata(String),
    Def { name: String, type_name: String },
    Codef { name: String, type_name: String },
}

impl Item {
    pub fn type_name(&self) -> &str {
        match self {
            Item::Data(name) => name,
            Item::Codata(name) => name,
            Item::Def { type_name, .. } => type_name,
            Item::Codef { type_name, .. } => type_name,
        }
    }
}
