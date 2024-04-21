use std::fmt;
use std::hash::Hash;
use std::rc::Rc;

use codespan::Span;
use derivative::Derivative;

use crate::common::*;

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

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Label {
    /// A machine-generated, unique id
    pub id: usize,
    /// A user-annotated name
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub user_name: Option<Ident>,
}

impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.user_name {
            None => Ok(()),
            Some(user_name) => user_name.fmt(f),
        }
    }
}

pub trait HasPhase {
    type Phase;
}

pub trait Named {
    fn name(&self) -> &Ident;
}

impl Named for Ident {
    fn name(&self) -> &Ident {
        self
    }
}

impl<'a, T> Named for &'a T
where
    T: Named,
{
    fn name(&self) -> &Ident {
        T::name(self)
    }
}

pub type Ident = String;

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
        body: Match<P>,
    },
    Hole {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        info: P::TypeInfo,
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
pub struct Match<P: Phase> {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub info: Option<Span>,
    pub cases: Vec<Case<P>>,
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
pub struct Motive<P: Phase> {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub info: Option<Span>,
    pub param: ParamInst<P>,
    pub ret_typ: Rc<Exp<P>>,
}
