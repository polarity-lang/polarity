use codespan::Span;
use printer::PrintToString;

use syntax::{
    ast::CallKind,
    ast::DotCallKind,
    ctx::values::{Binder as TypeCtxBinder, TypeCtx},
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
    // Toplevel Declarations
    DataInfo(DataInfo),
    CtorInfo(CtorInfo),
    CodataInfo(CodataInfo),
    DtorInfo(DtorInfo),
    DefInfo(DefInfo),
    CodefInfo(CodefInfo),
    LetInfo(LetInfo),
}

// Info structs for toplevel declarations
//
//

/// Information for toplevel data type declarations
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataInfo {
    /// Name of the data type
    pub name: String,
    /// Doc comments for data type
    pub doc: Option<Vec<String>>,
    /// Parameters
    pub params: String,
}

impl From<DataInfo> for InfoContent {
    fn from(value: DataInfo) -> Self {
        InfoContent::DataInfo(value)
    }
}

/// Information about constructor within a data type declaration
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CtorInfo {
    pub name: String,
    pub doc: Option<Vec<String>>,
}

impl From<CtorInfo> for InfoContent {
    fn from(value: CtorInfo) -> Self {
        InfoContent::CtorInfo(value)
    }
}

/// Information for toplevel codata type declarations
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodataInfo {
    /// Name of the data type
    pub name: String,
    /// Doc comments for data type
    pub doc: Option<Vec<String>>,
    /// Parameters
    pub params: String,
}

impl From<CodataInfo> for InfoContent {
    fn from(value: CodataInfo) -> Self {
        InfoContent::CodataInfo(value)
    }
}

/// Information about destructor within a data type declaration
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DtorInfo {
    pub name: String,
    pub doc: Option<Vec<String>>,
}

impl From<DtorInfo> for InfoContent {
    fn from(value: DtorInfo) -> Self {
        InfoContent::DtorInfo(value)
    }
}

/// Information for toplevel definitions
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefInfo {}

impl From<DefInfo> for InfoContent {
    fn from(value: DefInfo) -> Self {
        InfoContent::DefInfo(value)
    }
}

/// Information for toplevel codefinitions
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodefInfo {}

impl From<CodefInfo> for InfoContent {
    fn from(value: CodefInfo) -> Self {
        InfoContent::CodefInfo(value)
    }
}

/// Information for toplevel let bindings
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LetInfo {}

impl From<LetInfo> for InfoContent {
    fn from(value: LetInfo) -> Self {
        InfoContent::LetInfo(value)
    }
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
    pub args: String,
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
