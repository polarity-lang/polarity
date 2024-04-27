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
pub struct Prg<P: Phase> {
    pub decls: Decls<P>,
}

impl<P: Phase> Prg<P> {
    pub fn find_main(&self) -> Option<Rc<Exp<P>>> {
        let main_candidate = self.decls.map.get("main")?.get_main()?;
        Some(main_candidate.body)
    }
}

#[derive(Debug, Clone)]
pub struct Decls<P: Phase> {
    /// Map from identifiers to declarations
    pub map: HashMap<Ident, Decl<P>>,
    /// Metadata on declarations
    pub lookup_table: LookupTable,
}

#[derive(Debug, Clone)]
pub enum Decl<P: Phase> {
    Data(Data<P>),
    Codata(Codata<P>),
    Ctor(Ctor<P>),
    Dtor(Dtor<P>),
    Def(Def<P>),
    Codef(Codef<P>),
    Let(Let<P>),
}

impl<P: Phase> Decl<P> {
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
    pub fn get_main(&self) -> Option<Let<P>> {
        match self {
            Decl::Let(tl_let) => tl_let.is_main().then(|| tl_let.clone()),
            _ => None,
        }
    }
}

impl<P: Phase> Named for Decl<P> {
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

impl<P: Phase> HasSpan for Decl<P> {
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
pub struct Data<P: Phase> {
    pub span: Option<Span>,
    pub doc: Option<DocComment>,
    pub name: Ident,
    pub attr: Attribute,
    pub typ: Rc<TypAbs<P>>,
    pub ctors: Vec<Ident>,
}

#[derive(Debug, Clone)]
pub struct Codata<P: Phase> {
    pub span: Option<Span>,
    pub doc: Option<DocComment>,
    pub name: Ident,
    pub attr: Attribute,
    pub typ: Rc<TypAbs<P>>,
    pub dtors: Vec<Ident>,
}

#[derive(Debug, Clone)]
pub struct TypAbs<P: Phase> {
    pub params: Telescope<P>,
}

#[derive(Debug, Clone)]
pub struct Ctor<P: Phase> {
    pub span: Option<Span>,
    pub doc: Option<DocComment>,
    pub name: Ident,
    pub params: Telescope<P>,
    pub typ: TypCtor<P>,
}

#[derive(Debug, Clone)]
pub struct Dtor<P: Phase> {
    pub span: Option<Span>,
    pub doc: Option<DocComment>,
    pub name: Ident,
    pub params: Telescope<P>,
    pub self_param: SelfParam<P>,
    pub ret_typ: Rc<Exp<P>>,
}

#[derive(Debug, Clone)]
pub struct Def<P: Phase> {
    pub span: Option<Span>,
    pub doc: Option<DocComment>,
    pub name: Ident,
    pub attr: Attribute,
    pub params: Telescope<P>,
    pub self_param: SelfParam<P>,
    pub ret_typ: Rc<Exp<P>>,
    pub body: Match<P>,
}

impl<P: Phase> Def<P> {
    pub fn to_dtor(&self) -> Dtor<P> {
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
pub struct Codef<P: Phase> {
    pub span: Option<Span>,
    pub doc: Option<DocComment>,
    pub name: Ident,
    pub attr: Attribute,
    pub params: Telescope<P>,
    pub typ: TypCtor<P>,
    pub body: Match<P>,
}

impl<P: Phase> Codef<P> {
    pub fn to_ctor(&self) -> Ctor<P> {
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
pub struct Let<P: Phase> {
    pub span: Option<Span>,
    pub doc: Option<DocComment>,
    pub name: Ident,
    pub attr: Attribute,
    pub params: Telescope<P>,
    pub typ: Rc<Exp<P>>,
    pub body: Rc<Exp<P>>,
}

impl<P: Phase> Let<P> {
    /// Returns whether the declaration is the "main" expression of the module.
    pub fn is_main(&self) -> bool {
        self.name == "main" && self.params.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct SelfParam<P: Phase> {
    pub info: Option<Span>,
    pub name: Option<Ident>,
    pub typ: TypCtor<P>,
}

impl<P: Phase> SelfParam<P> {
    pub fn telescope(&self) -> Telescope<P> {
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
pub struct Telescope<P: Phase> {
    pub params: Vec<Param<P>>,
}

impl<P: Phase> Telescope<P> {
    pub fn len(&self) -> usize {
        self.params.len()
    }

    pub fn is_empty(&self) -> bool {
        self.params.is_empty()
    }
}

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Param<P: Phase> {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub name: Ident,
    pub typ: Rc<Exp<P>>,
}

impl<P: Phase> Named for Param<P> {
    fn name(&self) -> &Ident {
        &self.name
    }
}

impl<P: Phase> Named for ParamInst<P> {
    fn name(&self) -> &Ident {
        &self.name
    }
}

impl<P: Phase> HasPhase for Prg<P> {
    type Phase = P;
}

impl<P: Phase> HasPhase for Decls<P> {
    type Phase = P;
}

impl<P: Phase> HasPhase for Decl<P> {
    type Phase = P;
}

impl<P: Phase> HasPhase for Let<P> {
    type Phase = P;
}

impl<P: Phase> HasPhase for Data<P> {
    type Phase = P;
}

impl<P: Phase> HasPhase for Codata<P> {
    type Phase = P;
}

impl<P: Phase> HasPhase for TypAbs<P> {
    type Phase = P;
}

impl<P: Phase> HasPhase for Ctor<P> {
    type Phase = P;
}

impl<P: Phase> HasPhase for Dtor<P> {
    type Phase = P;
}

impl<P: Phase> HasPhase for Def<P> {
    type Phase = P;
}

impl<P: Phase> HasPhase for Codef<P> {
    type Phase = P;
}

impl<P: Phase> HasPhase for Match<P> {
    type Phase = P;
}

impl<P: Phase> HasPhase for Case<P> {
    type Phase = P;
}

impl<P: Phase> HasPhase for Exp<P> {
    type Phase = P;
}

impl<P: Phase> HasPhase for Telescope<P> {
    type Phase = P;
}

impl<P: Phase> HasPhase for Args<P> {
    type Phase = P;
}

impl<P: Phase> HasPhase for Param<P> {
    type Phase = P;
}

impl<T: HasPhase> HasPhase for Rc<T> {
    type Phase = T::Phase;
}
