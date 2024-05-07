use codespan::Span;
use printer::PrintToString;

use syntax::{
    ctx::values::{Binder as TypeCtxBinder, TypeCtx},
    generic::CallKind,
    generic::DotCallKind,
};

// Info
//
// Types which contain the information that is used by the LSP server
// to compute type-on-hover information and to provide the jump-to-definition
// functionality.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Info {
    /// The source code location to which the content applies
    pub span: Span,
    /// The information that should be displayed on hover
    pub content: InfoContent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InfoContent {
    VariableInfo(VariableInfo),
    TypeCtorInfo(TypeCtorInfo),
    CallInfo(CallInfo),
    DotCallInfo(DotCallInfo),
    TypeUnivInfo(TypeUnivInfo),
    HoleInfo(HoleInfo),
    AnnoInfo(AnnoInfo),
}

/// Information for bound variables
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariableInfo {
    pub typ: String,
}

impl From<VariableInfo> for InfoContent {
    fn from(value: VariableInfo) -> Self {
        InfoContent::VariableInfo(value)
    }
}

/// Information for type constructors
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeCtorInfo {}

impl From<TypeCtorInfo> for InfoContent {
    fn from(value: TypeCtorInfo) -> Self {
        InfoContent::TypeCtorInfo(value)
    }
}

/// Information for calls (constructors, codefinitions or lets)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallInfo {
    pub kind: CallKind,
    pub typ: String,
}

impl From<CallInfo> for InfoContent {
    fn from(value: CallInfo) -> Self {
        InfoContent::CallInfo(value)
    }
}

/// Information for dotcalls (destructors or definitions)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DotCallInfo {
    pub kind: DotCallKind,
    pub typ: String,
}

impl From<DotCallInfo> for InfoContent {
    fn from(value: DotCallInfo) -> Self {
        InfoContent::DotCallInfo(value)
    }
}

/// Information for type universes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeUnivInfo {}

impl From<TypeUnivInfo> for InfoContent {
    fn from(value: TypeUnivInfo) -> Self {
        InfoContent::TypeUnivInfo(value)
    }
}

/// Information for type annotated terms
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnnoInfo {
    pub typ: String,
}

impl From<AnnoInfo> for InfoContent {
    fn from(value: AnnoInfo) -> Self {
        InfoContent::AnnoInfo(value)
    }
}

/// Information for typed holes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HoleInfo {
    pub goal: String,
    pub ctx: Option<Ctx>,
}

impl From<HoleInfo> for InfoContent {
    fn from(value: HoleInfo) -> Self {
        InfoContent::HoleInfo(value)
    }
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
