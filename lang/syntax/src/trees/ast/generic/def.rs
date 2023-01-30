use std::fmt;
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
    /// Type of the `typ` field for `ParamInst`
    type Typ: Clone + fmt::Debug;

    fn print_var(name: &Self::VarName, idx: Option<Idx>) -> String;
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
            source::Item::Impl(impl_block) => {
                Item::Impl(self.map[&impl_block.name].impl_block().unwrap())
            }
            source::Item::Type(type_decl) => match &self.map[&type_decl.name] {
                Decl::Data(data) => Item::Data(data),
                Decl::Codata(codata) => Item::Codata(codata),
                _ => unreachable!(),
            },
        })
    }

    pub fn typ(&self, name: &str) -> Type<'_, P> {
        match &self.map[name] {
            Decl::Data(data) => Type::Data(data),
            Decl::Codata(codata) => Type::Codata(codata),
            _ => unreachable!(),
        }
    }

    pub fn type_decl_for_member(&self, name: &Ident) -> Type<'_, P> {
        let type_decl = self
            .source
            .type_decl_for_xtor(name)
            .unwrap_or_else(|| self.source.type_decl_for_xdef(name).unwrap());
        self.typ(&type_decl.name)
    }

    pub fn def(&self, name: &str) -> &Def<P> {
        match &self.map[name] {
            Decl::Def(def) => def,
            _ => unreachable!(),
        }
    }

    pub fn codef(&self, name: &str) -> &Codef<P> {
        match &self.map[name] {
            Decl::Codef(codef) => codef,
            _ => unreachable!(),
        }
    }

    pub fn ctor_or_codef(&self, name: &str) -> Ctor<P> {
        match &self.map[name] {
            Decl::Ctor(ctor) => ctor.clone(),
            Decl::Codef(codef) => codef.to_ctor(),
            _ => unreachable!(),
        }
    }

    pub fn dtor_or_def(&self, name: &str) -> Dtor<P> {
        match &self.map[name] {
            Decl::Dtor(dtor) => dtor.clone(),
            Decl::Def(def) => def.to_dtor(),
            _ => unreachable!(),
        }
    }

    pub fn ctor(&self, name: &str) -> &Ctor<P> {
        match &self.map[name] {
            Decl::Ctor(ctor) => ctor,
            _ => unreachable!(),
        }
    }

    pub fn dtor(&self, name: &str) -> &Dtor<P> {
        match &self.map[name] {
            Decl::Dtor(dtor) => dtor,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Item<'a, P: Phase> {
    Data(&'a Data<P>),
    Codata(&'a Codata<P>),
    Impl(&'a Impl<P>),
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

impl<P: Phase> Decl<P> {
    fn impl_block(&self) -> Option<&Impl<P>> {
        match self {
            Decl::Data(data) => data.impl_block.as_ref(),
            Decl::Codata(codata) => codata.impl_block.as_ref(),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Data<P: Phase> {
    pub info: P::Info,
    pub name: Ident,
    pub typ: Rc<TypAbs<P>>,
    pub ctors: Vec<Ident>,
    pub impl_block: Option<Impl<P>>,
}

#[derive(Debug, Clone)]
pub struct Codata<P: Phase> {
    pub info: P::Info,
    pub name: Ident,
    pub typ: Rc<TypAbs<P>>,
    pub dtors: Vec<Ident>,
    pub impl_block: Option<Impl<P>>,
}

#[derive(Debug, Clone)]
pub struct Impl<P: Phase> {
    pub info: P::Info,
    pub name: Ident,
    pub defs: Vec<Ident>,
}

#[derive(Debug, Clone)]
pub struct TypAbs<P: Phase> {
    pub params: Telescope<P>,
}

#[derive(Debug, Clone)]
pub struct Ctor<P: Phase> {
    pub info: P::Info,
    pub name: Ident,
    pub params: Telescope<P>,
    pub typ: TypApp<P>,
}

#[derive(Debug, Clone)]
pub struct Dtor<P: Phase> {
    pub info: P::Info,
    pub name: Ident,
    pub params: Telescope<P>,
    pub self_param: SelfParam<P>,
    pub ret_typ: Rc<Exp<P>>,
}

#[derive(Debug, Clone)]
pub struct Def<P: Phase> {
    pub info: P::Info,
    pub name: Ident,
    pub params: Telescope<P>,
    pub self_param: SelfParam<P>,
    pub ret_typ: Rc<Exp<P>>,
    pub body: Match<P>,
}

impl<P: Phase> Def<P> {
    pub fn to_dtor(&self) -> Dtor<P> {
        Dtor {
            info: self.info.clone(),
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
    pub name: Ident,
    pub params: Telescope<P>,
    pub typ: TypApp<P>,
    pub body: Comatch<P>,
}

impl<P: Phase> Codef<P> {
    pub fn to_ctor(&self) -> Ctor<P> {
        Ctor {
            info: self.info.clone(),
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
        name: Ident,
        on_exp: Rc<Exp<P>>,
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        ret_typ: P::Typ,
        // TODO: Ignore this field for PartialEq, Hash?
        body: Match<P>,
    },
    Comatch {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        info: P::TypeAppInfo,
        name: Ident,
        // TODO: Ignore this field for PartialEq, Hash?
        body: Comatch<P>,
    },
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
    pub typ: P::Typ,
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

impl<P: Phase> HasPhase for Impl<P> {
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
