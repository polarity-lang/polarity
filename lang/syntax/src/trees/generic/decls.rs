use std::rc::Rc;

use codespan::Span;
use derivative::Derivative;

use crate::common::*;

use super::lookup_table::{DeclKind, LookupTable};
use super::{exp::*, ForgetTST, Instantiate};

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

// Prg
//
//

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

impl ForgetTST for Prg {
    fn forget_tst(&self) -> Self {
        let Prg { decls } = self;

        Prg { decls: decls.forget_tst() }
    }
}

// Decls
//
//

#[derive(Debug, Clone)]
pub struct Decls {
    /// Map from identifiers to declarations
    pub map: HashMap<Ident, Decl>,
    /// Metadata on declarations
    pub lookup_table: LookupTable,
}

impl ForgetTST for Decls {
    fn forget_tst(&self) -> Self {
        let Decls { map, lookup_table } = self;

        Decls {
            map: map.iter().map(|(name, decl)| (name.clone(), decl.forget_tst())).collect(),
            lookup_table: lookup_table.clone(),
        }
    }
}

// Decl
//
//

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

impl ForgetTST for Decl {
    fn forget_tst(&self) -> Self {
        match self {
            Decl::Data(data) => Decl::Data(data.forget_tst()),
            Decl::Codata(codata) => Decl::Codata(codata.forget_tst()),
            Decl::Ctor(ctor) => Decl::Ctor(ctor.forget_tst()),
            Decl::Dtor(dtor) => Decl::Dtor(dtor.forget_tst()),
            Decl::Def(def) => Decl::Def(def.forget_tst()),
            Decl::Codef(codef) => Decl::Codef(codef.forget_tst()),
            Decl::Let(tl_let) => Decl::Let(tl_let.forget_tst()),
        }
    }
}

// Data
//
//

#[derive(Debug, Clone)]
pub struct Data {
    pub span: Option<Span>,
    pub doc: Option<DocComment>,
    pub name: Ident,
    pub attr: Attribute,
    pub typ: Rc<Telescope>,
    pub ctors: Vec<Ident>,
}

impl ForgetTST for Data {
    fn forget_tst(&self) -> Self {
        let Data { span, doc, name, attr, typ, ctors } = self;

        Data {
            span: *span,
            name: name.clone(),
            doc: doc.clone(),
            attr: attr.clone(),
            typ: typ.forget_tst(),
            ctors: ctors.clone(),
        }
    }
}

// Codata
//
//

#[derive(Debug, Clone)]
pub struct Codata {
    pub span: Option<Span>,
    pub doc: Option<DocComment>,
    pub name: Ident,
    pub attr: Attribute,
    pub typ: Rc<Telescope>,
    pub dtors: Vec<Ident>,
}

impl ForgetTST for Codata {
    fn forget_tst(&self) -> Self {
        let Codata { span, doc, name, attr, typ, dtors } = self;

        Codata {
            span: *span,
            doc: doc.clone(),
            name: name.clone(),
            attr: attr.clone(),
            typ: typ.forget_tst(),
            dtors: dtors.clone(),
        }
    }
}

// Ctor
//
//

#[derive(Debug, Clone)]
pub struct Ctor {
    pub span: Option<Span>,
    pub doc: Option<DocComment>,
    pub name: Ident,
    pub params: Telescope,
    pub typ: TypCtor,
}

impl ForgetTST for Ctor {
    fn forget_tst(&self) -> Self {
        let Ctor { span, doc, name, params, typ } = self;

        Ctor {
            span: *span,
            doc: doc.clone(),
            name: name.clone(),
            params: params.forget_tst(),
            typ: typ.forget_tst(),
        }
    }
}

// Dtor
//
//

#[derive(Debug, Clone)]
pub struct Dtor {
    pub span: Option<Span>,
    pub doc: Option<DocComment>,
    pub name: Ident,
    pub params: Telescope,
    pub self_param: SelfParam,
    pub ret_typ: Rc<Exp>,
}

impl ForgetTST for Dtor {
    fn forget_tst(&self) -> Self {
        let Dtor { span, doc, name, params, self_param, ret_typ } = self;

        Dtor {
            span: *span,
            doc: doc.clone(),
            name: name.clone(),
            params: params.forget_tst(),
            self_param: self_param.forget_tst(),
            ret_typ: ret_typ.forget_tst(),
        }
    }
}

// Def
//
//

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

impl ForgetTST for Def {
    fn forget_tst(&self) -> Self {
        let Def { span, doc, name, attr, params, self_param, ret_typ, body } = self;

        Def {
            span: *span,
            doc: doc.clone(),
            name: name.clone(),
            attr: attr.clone(),
            params: params.forget_tst(),
            self_param: self_param.forget_tst(),
            ret_typ: ret_typ.forget_tst(),
            body: body.forget_tst(),
        }
    }
}

// Codef
//
//

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

impl ForgetTST for Codef {
    fn forget_tst(&self) -> Self {
        let Codef { span, doc, name, attr, params, typ, body } = self;

        Codef {
            span: *span,
            doc: doc.clone(),
            name: name.clone(),
            attr: attr.clone(),
            params: params.forget_tst(),
            typ: typ.forget_tst(),
            body: body.forget_tst(),
        }
    }
}

// Let
//
//

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

impl ForgetTST for Let {
    fn forget_tst(&self) -> Self {
        let Let { span, doc, name, attr, params, typ, body } = self;

        Let {
            span: *span,
            doc: doc.clone(),
            name: name.clone(),
            attr: attr.clone(),
            params: params.forget_tst(),
            typ: typ.forget_tst(),
            body: body.forget_tst(),
        }
    }
}
// SelfParam
//
//

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

impl ForgetTST for SelfParam {
    fn forget_tst(&self) -> Self {
        let SelfParam { info, name, typ } = self;

        SelfParam { info: *info, name: name.clone(), typ: typ.forget_tst() }
    }
}

// Telescope
//
//

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
impl Instantiate for Telescope {
    fn instantiate(&self) -> TelescopeInst {
        let params = self
            .params
            .iter()
            .map(|Param { name, .. }| ParamInst {
                span: None,
                name: name.clone(),
                info: None,
                typ: None,
            })
            .collect();
        TelescopeInst { params }
    }
}

impl ForgetTST for Telescope {
    fn forget_tst(&self) -> Self {
        let Telescope { params } = self;

        Telescope { params: params.forget_tst() }
    }
}

// Param
//
//

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
impl ForgetTST for Param {
    fn forget_tst(&self) -> Self {
        let Param { name, typ } = self;

        Param { name: name.clone(), typ: typ.forget_tst() }
    }
}
