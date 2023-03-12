use std::fmt;
use std::hash::Hash;
use std::rc::Rc;

use data::HashMap;
use derivative::Derivative;

use crate::common::*;

use super::source::{self, Source};

pub trait Phase
where
    Self: Default + Clone + fmt::Debug + Eq,
{
    /// Type of the `info` field, containing span information
    type Info: HasSpan + Clone + fmt::Debug;
    /// Type of the `info` field, containing span and (depending on the phase) type information
    type TypeInfo: HasSpan + Clone + fmt::Debug;
    /// Type of the `info` field, containing span and (depending on the phase) type information
    /// where the type is required to be the full application of a type constructor
    type TypeAppInfo: HasSpan + Clone + Into<Self::TypeInfo> + fmt::Debug;
    /// Type of the `name` field of `Exp::Var`
    type VarName: Clone + fmt::Debug;
    /// Type of the `name` fiels on `Exp::Match` and `Exp::Comatch`
    type Label: Clone + Eq + Hash + fmt::Debug;
    /// A type which is not annotated in the source, but will be filled in later during typechecking
    type InfTyp: Clone + fmt::Debug;

    fn print_var(name: &Self::VarName, idx: Option<Idx>) -> String;
    fn print_label(name: &Self::Label) -> Option<String>;
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
    /// Order in which declarations are defined in the source
    pub source: Source,
}

impl<P: Phase> Decls<P> {
    pub fn empty() -> Self {
        Self { map: data::HashMap::default(), source: Default::default() }
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = Item<'_, P>> {
        self.source.iter().map(|item| match item {
            source::Item::Type(type_decl) => match &self.map[&type_decl.name] {
                Decl::Data(data) => Item::Data(data),
                Decl::Codata(codata) => Item::Codata(codata),
                _ => unreachable!(),
            },
            source::Item::Def(def_decl) => match &self.map[&def_decl.name] {
                Decl::Def(def) => Item::Def(def),
                Decl::Codef(codef) => Item::Codef(codef),
                _ => unreachable!(),
            },
        })
    }

    pub fn typ(&self, name: &str) -> Type<'_, P> {
        match &self.map[name] {
            Decl::Data(data) => Type::Data(data),
            Decl::Codata(codata) => Type::Codata(codata),
            Decl::Ctor(ctor) => panic!("Ctor {} unreachable", ctor.name),
            Decl::Dtor(dtor) => panic!("Dtor {} unreachable", dtor.name),
            Decl::Def(def) => panic!("Def {} unrechable", def.name),
            Decl::Codef(codef) => panic!("Codef {} unrechable", codef.name),
        }
    }

    pub fn type_decl_for_member(&self, name: &Ident) -> Type<'_, P> {
        let type_decl = self
            .source
            .type_decl_for_xtor(name)
            .unwrap_or_else(|| self.source.type_decl_for_xdef(name).unwrap());
        self.typ(&type_decl.name)
    }

    pub fn xtors_for_type(&self, name: &str) -> Vec<Ident> {
        self.source.xtors_for_type(name)
    }

    pub fn xdefs_for_type(&self, name: &str) -> Vec<Ident> {
        self.source.xdefs_for_type(name)
    }

    pub fn def(&self, name: &str) -> Option<&Def<P>> {
        match &self.map.get(name)? {
            Decl::Def(def) => Some(def),
            _ => None,
        }
    }

    pub fn codef(&self, name: &str) -> Option<&Codef<P>> {
        match &self.map.get(name)? {
            Decl::Codef(codef) => Some(codef),
            _ => None,
        }
    }

    pub fn ctor_or_codef(&self, name: &str) -> Option<Ctor<P>> {
        match &self.map.get(name)? {
            Decl::Ctor(ctor) => Some(ctor.clone()),
            Decl::Codef(codef) => Some(codef.to_ctor()),
            _ => None,
        }
    }

    pub fn dtor_or_def(&self, name: &str) -> Option<Dtor<P>> {
        match &self.map.get(name)? {
            Decl::Dtor(dtor) => Some(dtor.clone()),
            Decl::Def(def) => Some(def.to_dtor()),
            _ => None,
        }
    }

    pub fn ctor(&self, name: &str) -> Option<&Ctor<P>> {
        match &self.map.get(name)? {
            Decl::Ctor(ctor) => Some(ctor),
            _ => None,
        }
    }

    pub fn dtor(&self, name: &str) -> Option<&Dtor<P>> {
        match &self.map.get(name)? {
            Decl::Dtor(dtor) => Some(dtor),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Item<'a, P: Phase> {
    Data(&'a Data<P>),
    Codata(&'a Codata<P>),
    Def(&'a Def<P>),
    Codef(&'a Codef<P>),
}

impl<'a, P: Phase> Item<'a, P> {
    pub fn hidden(&self) -> bool {
        match self {
            Item::Data(data) => data.hidden,
            Item::Codata(codata) => codata.hidden,
            Item::Def(def) => def.hidden,
            Item::Codef(codef) => codef.hidden,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Type<'a, P: Phase> {
    Data(&'a Data<P>),
    Codata(&'a Codata<P>),
}

impl<'a, P: Phase> Type<'a, P> {
    pub fn name(&self) -> &Ident {
        match self {
            Type::Data(data) => &data.name,
            Type::Codata(codata) => &codata.name,
        }
    }

    pub fn typ(&self) -> Rc<TypAbs<P>> {
        match self {
            Type::Data(data) => data.typ.clone(),
            Type::Codata(codata) => codata.typ.clone(),
        }
    }
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

#[derive(Debug, Clone)]
pub struct Data<P: Phase> {
    pub info: P::Info,
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
    pub info: P::Info,
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
    pub info: P::Info,
    pub doc: Option<DocComment>,
    pub name: Ident,
    pub params: Telescope<P>,
    pub typ: TypApp<P>,
}

#[derive(Debug, Clone)]
pub struct Dtor<P: Phase> {
    pub info: P::Info,
    pub doc: Option<DocComment>,
    pub name: Ident,
    pub params: Telescope<P>,
    pub self_param: SelfParam<P>,
    pub ret_typ: Rc<Exp<P>>,
}

#[derive(Debug, Clone)]
pub struct Def<P: Phase> {
    pub info: P::Info,
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
            info: self.info.clone(),
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
    pub info: P::Info,
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
            info: self.info.clone(),
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
    pub info: P::Info,
    pub cases: Vec<Case<P>>,
}

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Comatch<P: Phase> {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub info: P::Info,
    // TODO: Consider renaming this field to "cocases"
    pub cases: Vec<Cocase<P>>,
}

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Case<P: Phase> {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub info: P::Info,
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
    pub info: P::Info,
    pub name: Ident,
    pub params: TelescopeInst<P>,
    /// Body being `None` represents an absurd pattern
    pub body: Option<Rc<Exp<P>>>,
}

#[derive(Debug, Clone)]
pub struct SelfParam<P: Phase> {
    pub info: P::Info,
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
        name: P::VarName,
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
        name: P::Label,
        on_exp: Rc<Exp<P>>,
        motive: Option<Motive<P>>,
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        ret_typ: P::InfTyp,
        body: Match<P>,
    },
    Comatch {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        info: P::TypeAppInfo,
        name: P::Label,
        body: Comatch<P>,
    },
    Hole {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        info: P::TypeInfo,
    },
}

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Motive<P: Phase> {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub info: P::Info,
    pub param: ParamInst<P>,
    pub ret_typ: Rc<Exp<P>>,
}

/// Wrapper type signifying the wrapped parameters have telescope
/// semantics. I.e. each parameter binding in the parameter list is in scope
/// for the following parameters.
#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Telescope<P: Phase> {
    pub params: Params<P>,
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

pub type Params<P> = Vec<Param<P>>;
pub type Args<P> = Vec<Rc<Exp<P>>>;

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

impl<P: Phase> HasPhase for Params<P> {
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
