use std::rc::Rc;

use codespan::Span;
use derivative::Derivative;

use crate::common::*;

use super::exp::*;
use super::lookup_table::{DeclKind, LookupTable};

#[derive(Debug, Clone)]
pub struct DocComment {
    pub docs: Vec<String>,
}

/// An attribute can be attached to various nodes in the syntax tree.
/// We use the same syntax for attributes as Rust, that is `#[attr1,attr2]`.
#[derive(Debug, Clone, Default)]
pub struct Attribute {
    pub attrs: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Prg {
    pub decls: Decls,
}

impl Prg {
    pub fn find_main(&self) -> Option<Rc<Exp>> {
        let main_candidate = self.decls.map.get("main")?.get_main()?;
        Some(main_candidate.body)
    }
}

#[derive(Debug, Clone)]
pub struct Decls {
    /// Map from identifiers to declarations
    pub map: HashMap<Ident, Decl>,
    /// Metadata on declarations
    pub lookup_table: LookupTable,
}

#[derive(Debug, Clone)]
pub enum Decl {
    Data(Data),
    Codata(Codata),
    Ctor(Ctor),
    Dtor(Dtor),
    Def(Def),
    Codef(Codef),
    Let(Let),
}

impl Decl {
    pub fn kind(&self) -> DeclKind {
        match self {
            Decl::Data(_) => DeclKind::Data,
            Decl::Codata(_) => DeclKind::Codata,
            Decl::Ctor(_) => DeclKind::Ctor,
            Decl::Dtor(_) => DeclKind::Dtor,
            Decl::Def(_) => DeclKind::Def,
            Decl::Codef(_) => DeclKind::Codef,
            Decl::Let(_) => DeclKind::Let,
        }
    }

    /// Returns whether the declaration is the "main" expression of the module.
    pub fn get_main(&self) -> Option<Let> {
        match self {
            Decl::Let(tl_let) => tl_let.is_main().then(|| tl_let.clone()),
            _ => None,
        }
    }
}

impl Named for Decl {
    fn name(&self) -> &Ident {
        match self {
            Decl::Data(Data { name, .. }) => name,
            Decl::Codata(Codata { name, .. }) => name,
            Decl::Def(Def { name, .. }) => name,
            Decl::Codef(Codef { name, .. }) => name,
            Decl::Ctor(Ctor { name, .. }) => name,
            Decl::Dtor(Dtor { name, .. }) => name,
            Decl::Let(Let { name, .. }) => name,
        }
    }
}

impl HasSpan for Decl {
    fn span(&self) -> Option<Span> {
        match self {
            Decl::Data(data) => data.span,
            Decl::Codata(codata) => codata.span,
            Decl::Ctor(ctor) => ctor.span,
            Decl::Dtor(dtor) => dtor.span,
            Decl::Def(def) => def.span,
            Decl::Codef(codef) => codef.span,
            Decl::Let(tl_let) => tl_let.span,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Data {
    pub span: Option<Span>,
    pub doc: Option<DocComment>,
    pub name: Ident,
    pub attr: Attribute,
    pub typ: Rc<TypAbs>,
    pub ctors: Vec<Ident>,
}

#[derive(Debug, Clone)]
pub struct Codata {
    pub span: Option<Span>,
    pub doc: Option<DocComment>,
    pub name: Ident,
    pub attr: Attribute,
    pub typ: Rc<TypAbs>,
    pub dtors: Vec<Ident>,
}

#[derive(Debug, Clone)]
pub struct TypAbs {
    pub params: Telescope,
}

#[derive(Debug, Clone)]
pub struct Ctor {
    pub span: Option<Span>,
    pub doc: Option<DocComment>,
    pub name: Ident,
    pub params: Telescope,
    pub typ: TypCtor,
}

#[derive(Debug, Clone)]
pub struct Dtor {
    pub span: Option<Span>,
    pub doc: Option<DocComment>,
    pub name: Ident,
    pub params: Telescope,
    pub self_param: SelfParam,
    pub ret_typ: Rc<Exp>,
}

#[derive(Debug, Clone)]
pub struct Def {
    pub span: Option<Span>,
    pub doc: Option<DocComment>,
    pub name: Ident,
    pub attr: Attribute,
    pub params: Telescope,
    pub self_param: SelfParam,
    pub ret_typ: Rc<Exp>,
    pub body: Match,
}

impl Def {
    pub fn to_dtor(&self) -> Dtor {
        Dtor {
            span: self.span,
            doc: self.doc.clone(),
            name: self.name.clone(),
            params: self.params.clone(),
            self_param: self.self_param.clone(),
            ret_typ: self.ret_typ.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Codef {
    pub span: Option<Span>,
    pub doc: Option<DocComment>,
    pub name: Ident,
    pub attr: Attribute,
    pub params: Telescope,
    pub typ: TypCtor,
    pub body: Match,
}

impl Codef {
    pub fn to_ctor(&self) -> Ctor {
        Ctor {
            span: self.span,
            doc: self.doc.clone(),
            name: self.name.clone(),
            params: self.params.clone(),
            typ: self.typ.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Let {
    pub span: Option<Span>,
    pub doc: Option<DocComment>,
    pub name: Ident,
    pub attr: Attribute,
    pub params: Telescope,
    pub typ: Rc<Exp>,
    pub body: Rc<Exp>,
}

impl Let {
    /// Returns whether the declaration is the "main" expression of the module.
    pub fn is_main(&self) -> bool {
        self.name == "main" && self.params.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct SelfParam {
    pub info: Option<Span>,
    pub name: Option<Ident>,
    pub typ: TypCtor,
}

impl SelfParam {
    pub fn telescope(&self) -> Telescope {
        Telescope {
            params: vec![Param {
                name: self.name.clone().unwrap_or_default(),
                typ: Rc::new(self.typ.to_exp()),
            }],
        }
    }

    /// A self parameter is simple if the list of arguments to the type is empty, and the name is None.
    /// If the self parameter is simple, we can omit it during prettyprinting.
    pub fn is_simple(&self) -> bool {
        self.typ.is_simple() && self.name.is_none()
    }
}

/// Wrapper type signifying the wrapped parameters have telescope
/// semantics. I.e. each parameter binding in the parameter list is in scope
/// for the following parameters.
#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Telescope {
    pub params: Vec<Param>,
}

impl Telescope {
    pub fn len(&self) -> usize {
        self.params.len()
    }

    pub fn is_empty(&self) -> bool {
        self.params.is_empty()
    }
}

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Param {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub name: Ident,
    pub typ: Rc<Exp>,
}

impl Named for Param {
    fn name(&self) -> &Ident {
        &self.name
    }
}

impl Named for ParamInst {
    fn name(&self) -> &Ident {
        &self.name
    }
}
