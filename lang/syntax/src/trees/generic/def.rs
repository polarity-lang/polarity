use std::fmt;
use std::hash::Hash;
use std::rc::Rc;

use codespan::Span;
use data::HashMap;
use derivative::Derivative;

use crate::common::*;
use parser::cst::{DocComment, HoleKind, Ident};

use super::lookup_table::{DeclKind, LookupTable};

pub trait Phase
where
    Self: Default + Clone + fmt::Debug + Eq,
{
    /// Type of the `info` field, containing span and (depending on the phase) type information
    type TypeInfo: HasSpan + Clone + fmt::Debug;
    /// Type of the `info` field, containing span and (depending on the phase) type information
    /// where the type is required to be the full application of a type constructor
    type TypeAppInfo: HasSpan + Clone + Into<Self::TypeInfo> + fmt::Debug;
    /// A type which is not annotated in the source, but will be filled in later during typechecking
    type InfTyp: Clone + fmt::Debug;
    /// Context annotated during typechecking
    type Ctx: Clone + fmt::Debug;
}

pub trait HasPhase {
    type Phase;
}

#[derive(Debug, Clone)]
pub struct Prg<P: Phase> {
    pub decls: Decls<P>,
    pub exp: Option<Rc<Exp<P>>>,
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
        }
    }
}
impl<P: Phase> HasSpan for Decl<P> {
    fn span(&self) -> Option<Span> {
        match self {
            Decl::Data(data) => data.info,
            Decl::Codata(codata) => codata.info,
            Decl::Ctor(ctor) => ctor.info,
            Decl::Dtor(dtor) => dtor.info,
            Decl::Def(def) => def.info,
            Decl::Codef(codef) => codef.info,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Data<P: Phase> {
    pub info: Option<Span>,
    pub doc: Option<DocComment>,
    pub name: Ident,
    /// Whether the declarations should be omitted
    /// during pretty printing.
    pub hidden: bool,
    pub typ: Rc<TypAbs<P>>,
    pub ctors: Vec<Ident>,
}

#[derive(Debug, Clone)]
pub struct Codata<P: Phase> {
    pub info: Option<Span>,
    pub doc: Option<DocComment>,
    pub name: Ident,
    /// Whether the declarations should be omitted
    /// during pretty printing.
    pub hidden: bool,
    pub typ: Rc<TypAbs<P>>,
    pub dtors: Vec<Ident>,
}

#[derive(Debug, Clone)]
pub struct TypAbs<P: Phase> {
    pub params: Telescope<P>,
}

#[derive(Debug, Clone)]
pub struct Ctor<P: Phase> {
    pub info: Option<Span>,
    pub doc: Option<DocComment>,
    pub name: Ident,
    pub params: Telescope<P>,
    pub typ: TypApp<P>,
}

#[derive(Debug, Clone)]
pub struct Dtor<P: Phase> {
    pub info: Option<Span>,
    pub doc: Option<DocComment>,
    pub name: Ident,
    pub params: Telescope<P>,
    pub self_param: SelfParam<P>,
    pub ret_typ: Rc<Exp<P>>,
}

#[derive(Debug, Clone)]
pub struct Def<P: Phase> {
    pub info: Option<Span>,
    pub doc: Option<DocComment>,
    pub name: Ident,
    /// Whether the declarations should be omitted
    /// during pretty printing.
    pub hidden: bool,
    pub params: Telescope<P>,
    pub self_param: SelfParam<P>,
    pub ret_typ: Rc<Exp<P>>,
    pub body: Match<P>,
}

impl<P: Phase> Def<P> {
    pub fn to_dtor(&self) -> Dtor<P> {
        Dtor {
            info: self.info,
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
    pub info: Option<Span>,
    pub doc: Option<DocComment>,
    pub name: Ident,
    /// Whether the declarations should be omitted
    /// during pretty printing.
    pub hidden: bool,
    pub params: Telescope<P>,
    pub typ: TypApp<P>,
    pub body: Comatch<P>,
}

impl<P: Phase> Codef<P> {
    pub fn to_ctor(&self) -> Ctor<P> {
        Ctor {
            info: self.info,
            doc: self.doc.clone(),
            name: self.name.clone(),
            params: self.params.clone(),
            typ: self.typ.clone(),
        }
    }
}

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Match<P: Phase> {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub info: Option<Span>,
    pub cases: Vec<Case<P>>,
    pub omit_absurd: bool,
}

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Comatch<P: Phase> {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub info: Option<Span>,
    // TODO: Consider renaming this field to "cocases"
    pub cases: Vec<Cocase<P>>,
    pub omit_absurd: bool,
}

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Case<P: Phase> {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub info: Option<Span>,
    pub name: Ident,
    // TODO: Rename to params
    pub args: TelescopeInst<P>,
    /// Body being `None` represents an absurd pattern
    pub body: Option<Rc<Exp<P>>>,
}

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Cocase<P: Phase> {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub info: Option<Span>,
    pub name: Ident,
    pub params: TelescopeInst<P>,
    /// Body being `None` represents an absurd pattern
    pub body: Option<Rc<Exp<P>>>,
}

#[derive(Debug, Clone)]
pub struct SelfParam<P: Phase> {
    pub info: Option<Span>,
    pub name: Option<Ident>,
    pub typ: TypApp<P>,
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

#[derive(Debug, Clone)]
pub struct TypApp<P: Phase> {
    pub info: P::TypeInfo,
    pub name: Ident,
    pub args: Args<P>,
}

impl<P: Phase> TypApp<P> {
    pub fn to_exp(&self) -> Exp<P> {
        Exp::TypCtor { info: self.info.clone(), name: self.name.clone(), args: self.args.clone() }
    }

    /// A type application is simple if the list of arguments is empty.
    pub fn is_simple(&self) -> bool {
        self.args.is_empty()
    }
}

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub enum Exp<P: Phase> {
    Var {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        info: P::TypeInfo,
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        name: Ident,
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        ctx: P::Ctx,
        idx: Idx,
    },
    TypCtor {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        info: P::TypeInfo,
        name: Ident,
        args: Args<P>,
    },
    Ctor {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        info: P::TypeInfo,
        name: Ident,
        args: Args<P>,
    },
    Dtor {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        info: P::TypeInfo,
        exp: Rc<Exp<P>>,
        name: Ident,
        args: Args<P>,
    },
    Anno {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        info: P::TypeInfo,
        exp: Rc<Exp<P>>,
        typ: Rc<Exp<P>>,
    },
    Type {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        info: P::TypeInfo,
    },
    Match {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        info: P::TypeAppInfo,
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        ctx: P::Ctx,
        name: Label,
        on_exp: Rc<Exp<P>>,
        motive: Option<Motive<P>>,
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        ret_typ: P::InfTyp,
        body: Match<P>,
    },
    Comatch {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        info: P::TypeAppInfo,
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        ctx: P::Ctx,
        name: Label,
        is_lambda_sugar: bool,
        body: Comatch<P>,
    },
    Hole {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        info: P::TypeInfo,
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        kind: HoleKind,
    },
}

impl<P: Phase> HasSpan for Exp<P> {
    fn span(&self) -> Option<Span> {
        match self {
            Exp::Var { info, .. } => info.span(),
            Exp::TypCtor { info, .. } => info.span(),
            Exp::Ctor { info, .. } => info.span(),
            Exp::Dtor { info, .. } => info.span(),
            Exp::Anno { info, .. } => info.span(),
            Exp::Type { info } => info.span(),
            Exp::Match { info, .. } => info.span(),
            Exp::Comatch { info, .. } => info.span(),
            Exp::Hole { info, .. } => info.span(),
        }
    }
}

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Motive<P: Phase> {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub info: Option<Span>,
    pub param: ParamInst<P>,
    pub ret_typ: Rc<Exp<P>>,
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

/// Instantiation of a previously declared telescope
#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct TelescopeInst<P: Phase> {
    pub params: Vec<ParamInst<P>>,
}

impl<P: Phase> TelescopeInst<P> {
    pub fn len(&self) -> usize {
        self.params.len()
    }

    pub fn is_empty(&self) -> bool {
        self.params.is_empty()
    }
}

/// A list of arguments
/// In dependent type theory, this concept is usually called a "substitution" but that name would be confusing in this implementation
/// because it conflicts with the operation of substitution (i.e. substituting a terms for a variable in another term).
/// In particular, while we often substitute argument lists for telescopes, this is not always the case.
/// Substitutions in the sense of argument lists are a special case of a more general concept of context morphisms.
/// Unifiers are another example of context morphisms and applying a unifier to an expression mean substituting various terms,
/// which are not necessarily part of a single argument list.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Args<P: Phase> {
    pub args: Vec<Rc<Exp<P>>>,
}

impl<P: Phase> Args<P> {
    pub fn len(&self) -> usize {
        self.args.len()
    }

    pub fn is_empty(&self) -> bool {
        self.args.is_empty()
    }
}

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Param<P: Phase> {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub name: Ident,
    pub typ: Rc<Exp<P>>,
}

/// Instantiation of a previously declared parameter
#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct ParamInst<P: Phase> {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub info: P::TypeInfo,
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub name: Ident,
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub typ: P::InfTyp,
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

impl<P: Phase> HasPhase for Comatch<P> {
    type Phase = P;
}

impl<P: Phase> HasPhase for Case<P> {
    type Phase = P;
}

impl<P: Phase> HasPhase for Cocase<P> {
    type Phase = P;
}

impl<P: Phase> HasPhase for TypApp<P> {
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
