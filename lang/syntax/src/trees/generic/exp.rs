use std::fmt;
use std::hash::Hash;
use std::rc::Rc;

use codespan::Span;
use derivative::Derivative;

use crate::common::*;
use crate::ctx::values::TypeCtx;
use crate::ust::UST;

pub trait Phase
where
    Self: Default + Clone + fmt::Debug + Eq,
{
    /// Type of the `info` field, containing (depending on the phase) type information
    type TypeInfo: Clone + fmt::Debug;
    /// Type of the `info` field, containing (depending on the phase) type information
    /// where the type is required to be the full application of a type constructor
    type TypeAppInfo: Clone + fmt::Debug;
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
    Variable(Variable),
    TypCtor(TypCtor<P>),
    Call(Call<P>),
    DotCall(DotCall<P>),
    Anno(Anno<P>),
    TypeUniv(TypeUniv),
    LocalMatch(LocalMatch<P>),
    LocalComatch(LocalComatch<P>),
    Hole(Hole),
}

impl<P: Phase> HasSpan for Exp<P> {
    fn span(&self) -> Option<Span> {
        match self {
            Exp::Variable(e) => e.span(),
            Exp::TypCtor(e) => e.span(),
            Exp::Call(e) => e.span(),
            Exp::DotCall(e) => e.span(),
            Exp::Anno(e) => e.span(),
            Exp::TypeUniv(e) => e.span(),
            Exp::LocalMatch(e) => e.span(),
            Exp::LocalComatch(e) => e.span(),
            Exp::Hole(e) => e.span(),
        }
    }
}

// Variable
//
//

/// A bound variable occurrence. The variable is represented
/// using a de-Bruijn index, but we keep the information
/// about the name that was originally annotated in the program.
#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Variable {
    /// Source code location
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub span: Option<Span>,
    /// The de-Bruijn index that is used to represent the
    /// binding structure of terms.
    pub idx: Idx,
    /// The name that was originally annotated in the program
    /// We do not use this information for tracking the binding
    /// structure, but only for prettyprinting code.
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub name: Ident,
    /// Inferred type annotated after elaboration.
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub inferred_type: Option<Rc<Exp<UST>>>,
}

impl HasSpan for Variable {
    fn span(&self) -> Option<Span> {
        self.span
    }
}

// TypCtor
//
//

/// A type constructor applied to arguments. The type of `TypCtor`
/// is always the type universe `Type`.
/// Examples: `Nat`, `List(Nat)`
#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct TypCtor<P: Phase> {
    /// Source code location
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub span: Option<Span>,
    /// Name of the type constructor
    pub name: Ident,
    /// Arguments to the type constructor
    pub args: Args<P>,
}

impl<P: Phase> TypCtor<P> {
    pub fn to_exp(&self) -> Exp<P> {
        Exp::TypCtor(self.clone())
    }

    /// A type application is simple if the list of arguments is empty.
    pub fn is_simple(&self) -> bool {
        self.args.is_empty()
    }
}

impl<P: Phase> HasSpan for TypCtor<P> {
    fn span(&self) -> Option<Span> {
        self.span
    }
}

// Call
//
//

/// A Call is either a constructor, a codefinition or a toplevel let-bound definition.
/// Examples: `Zero`, `Cons(True, Nil)`, `minimum(x,y)`
#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Call<P: Phase> {
    /// Source code location
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub span: Option<Span>,
    /// The name of the call.
    /// The `f` in `f(e1...en)`
    pub name: Ident,
    /// The arguments to the call.
    /// The `(e1...en)` in `f(e1...en)`
    pub args: Args<P>,
    /// The inferred result type of the call.
    /// This type is annotated during elaboration.
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub inferred_type: Option<Rc<Exp<UST>>>,
}

impl<P: Phase> HasSpan for Call<P> {
    fn span(&self) -> Option<Span> {
        self.span
    }
}

// DotCall
//
//

/// A DotCall is either a destructor or a definition, applied to a destructee
/// Examples: `e.head` `xs.append(ys)`
#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct DotCall<P: Phase> {
    /// Source code location
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub span: Option<Span>,
    /// The expression to which the dotcall is applied.
    /// The `e` in `e.f(e1...en)`
    pub exp: Rc<Exp<P>>,
    /// The name of the dotcall.
    /// The `f` in `e.f(e1...en)`
    pub name: Ident,
    /// The arguments of the dotcall.
    /// The `(e1...en)` in `e.f(e1...en)`
    pub args: Args<P>,
    /// The inferred result type of the dotcall.
    /// This type is annotated during elaboration.
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub inferred_type: Option<Rc<Exp<UST>>>,
}

impl<P: Phase> HasSpan for DotCall<P> {
    fn span(&self) -> Option<Span> {
        self.span
    }
}

// Anno
//
//

/// Type annotated term `e : t`
#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Anno<P: Phase> {
    /// Source code location
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub span: Option<Span>,
    /// The annotated term, i.e. `e` in `e : t`
    pub exp: Rc<Exp<P>>,
    /// The annotated type, i.e. `t` in `e : t`
    pub typ: Rc<Exp<P>>,
    /// The annotated type written by the user might not
    /// be in normal form. After elaboration we therefore
    /// annotate the normalized type.
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub normalized_type: Option<Rc<Exp<UST>>>,
}

impl<P: Phase> HasSpan for Anno<P> {
    fn span(&self) -> Option<Span> {
        self.span
    }
}

// TypeUniv
//
//

/// The impredicative type universe "Type" is used
/// for typing data and codata types. I.e. we have
/// - `Nat : Type`
/// - `Stream(Nat) : Type`
/// - `Type : Type`
#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct TypeUniv {
    /// Source code location
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub span: Option<Span>,
}

impl HasSpan for TypeUniv {
    fn span(&self) -> Option<Span> {
        self.span
    }
}

// LocalMatch
//
//

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct LocalMatch<P: Phase> {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub span: Option<Span>,
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub info: P::TypeAppInfo,
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub ctx: Option<TypeCtx>,
    pub name: Label,
    pub on_exp: Rc<Exp<P>>,
    pub motive: Option<Motive<P>>,
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub ret_typ: Option<Rc<Exp<P>>>,
    pub body: Match<P>,
}

impl<P: Phase> HasSpan for LocalMatch<P> {
    fn span(&self) -> Option<Span> {
        self.span
    }
}

// LocalComatch
//
//

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct LocalComatch<P: Phase> {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub span: Option<Span>,
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub info: P::TypeAppInfo,
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub ctx: Option<TypeCtx>,
    pub name: Label,
    pub is_lambda_sugar: bool,
    pub body: Match<P>,
}

impl<P: Phase> HasSpan for LocalComatch<P> {
    fn span(&self) -> Option<Span> {
        self.span
    }
}

// Hole
//
//

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Hole {
    /// Source code location
    pub span: Option<Span>,
    /// The inferred type of the hole annotated during elaboration.
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub inferred_type: Option<Rc<Exp<UST>>>,
    /// The type context in which the hole occurs.
    /// This context is annotated during elaboration.
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub inferred_ctx: Option<TypeCtx>,
}

impl HasSpan for Hole {
    fn span(&self) -> Option<Span> {
        self.span
    }
}

// Other
//
//

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Match<P: Phase> {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub span: Option<Span>,
    pub cases: Vec<Case<P>>,
    pub omit_absurd: bool,
}

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Case<P: Phase> {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub span: Option<Span>,
    pub name: Ident,
    pub params: TelescopeInst<P>,
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
    pub span: Option<Span>,
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub info: P::TypeInfo,
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub name: Ident,
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub typ: Option<Rc<Exp<P>>>,
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
    pub span: Option<Span>,
    pub param: ParamInst<P>,
    pub ret_typ: Rc<Exp<P>>,
}
