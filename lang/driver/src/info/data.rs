use miette_util::codespan::Span;
use printer::Print;

use ast::{
    ctx::values::{Binder as TypeCtxBinder, TypeCtx},
    CallKind, DotCallKind,
};
use url::Url;

// Info
//
// Types which contain information about source code locations
// that can be used by the LSP server to provide various services,
// such as type-on-hover and jump-to-definition.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Info {
    /// The source code location to which the content applies
    pub span: Span,
    /// The information that is available for that span
    pub content: InfoContent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InfoContent {
    // Expressions
    VariableInfo(VariableInfo),
    TypeCtorInfo(TypeCtorInfo),
    CallInfo(CallInfo),
    DotCallInfo(DotCallInfo),
    TypeUnivInfo(TypeUnivInfo),
    HoleInfo(HoleInfo),
    AnnoInfo(AnnoInfo),
    LocalMatchInfo(LocalMatchInfo),
    LocalComatchInfo(LocalComatchInfo),
    // Various
    CaseInfo(CaseInfo),
    PatternInfo(PatternInfo),
    // Modules
    UseInfo(UseInfo),
}

// Info structs for modules
//
//

/// Information for `use` statements
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UseInfo {
    /// The path as it appears in the source code
    pub path: String,
    /// The URI of the module that is being used
    pub uri: Url,
}

// Info structs for expressions
//
//

/// Information for bound variables
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariableInfo {
    pub name: String,
    pub typ: String,
}

impl From<VariableInfo> for InfoContent {
    fn from(value: VariableInfo) -> Self {
        InfoContent::VariableInfo(value)
    }
}

/// Information for type constructors
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeCtorInfo {
    /// The span where the type constructor was defined.
    /// This is used for the jump-to-definition feature.
    pub definition_site: Option<(Url, Span)>,
    /// Doc comment from the definition site
    pub doc: Option<Vec<String>>,
    pub name: String,
}

impl From<TypeCtorInfo> for InfoContent {
    fn from(value: TypeCtorInfo) -> Self {
        InfoContent::TypeCtorInfo(value)
    }
}

/// Information for calls (constructors, codefinitions or lets)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallInfo {
    /// The span where the call was defined.
    /// This is used for the jump-to-definition feature.
    pub definition_site: Option<(Url, Span)>,
    /// Doc comment from the definition site
    pub doc: Option<Vec<String>>,
    pub kind: CallKind,
    pub name: String,
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
    /// The span where the dotcall was defined.
    /// This is used for the jump-to-definition feature.
    pub definition_site: Option<(Url, Span)>,
    /// Doc comment from the definition site
    pub doc: Option<Vec<String>>,
    pub kind: DotCallKind,
    pub name: String,
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

// Information for patterns and copatterns
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatternInfo {
    pub name: String,
    pub is_copattern: bool,
}

impl From<PatternInfo> for InfoContent {
    fn from(value: PatternInfo) -> Self {
        InfoContent::PatternInfo(value)
    }
}

// Information for clauses in pattern and copattern matches
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaseInfo {}

impl From<CaseInfo> for InfoContent {
    fn from(value: CaseInfo) -> Self {
        InfoContent::CaseInfo(value)
    }
}

/// Information for local matches
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalMatchInfo {
    pub typ: String,
}

impl From<LocalMatchInfo> for InfoContent {
    fn from(value: LocalMatchInfo) -> Self {
        InfoContent::LocalMatchInfo(value)
    }
}

/// Information for local comatches
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalComatchInfo {
    pub typ: String,
}

impl From<LocalComatchInfo> for InfoContent {
    fn from(value: LocalComatchInfo) -> Self {
        InfoContent::LocalComatchInfo(value)
    }
}

/// Information for typed holes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HoleInfo {
    pub goal: String,
    pub metavar: Option<String>,
    pub ctx: Option<Ctx>,
    pub args: Vec<Vec<String>>,
    /// `Some(e)` if the solution`e` has been found for the metavariable.
    pub metavar_state: Option<String>,
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
pub enum Binder {
    Var { name: String, typ: String },
    Wildcard { typ: String },
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
        match binder.name {
            ast::VarBind::Var { id, .. } => {
                Binder::Var { name: id, typ: binder.typ.print_to_string(None) }
            }
            ast::VarBind::Wildcard { .. } => {
                Binder::Wildcard { typ: binder.typ.print_to_string(None) }
            }
        }
    }
}
