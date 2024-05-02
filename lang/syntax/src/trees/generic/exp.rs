use std::fmt;
use std::hash::Hash;
use std::rc::Rc;

use codespan::Span;
use derivative::Derivative;

use crate::common::*;
use crate::ctx::values::TypeCtx;

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
pub enum Exp {
    Variable(Variable),
    TypCtor(TypCtor),
    Call(Call),
    DotCall(DotCall),
    Anno(Anno),
    TypeUniv(TypeUniv),
    LocalMatch(LocalMatch),
    LocalComatch(LocalComatch),
    Hole(Hole),
}

impl HasSpan for Exp {
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

impl Shift for Exp {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        match self {
            Exp::Variable(e) => Exp::Variable(e.shift_in_range(range, by)),
            Exp::TypCtor(e) => Exp::TypCtor(e.shift_in_range(range, by)),
            Exp::Call(e) => Exp::Call(e.shift_in_range(range, by)),
            Exp::DotCall(e) => Exp::DotCall(e.shift_in_range(range, by)),
            Exp::Anno(e) => Exp::Anno(e.shift_in_range(range, by)),
            Exp::TypeUniv(e) => Exp::TypeUniv(e.shift_in_range(range, by)),
            Exp::LocalMatch(e) => Exp::LocalMatch(e.shift_in_range(range, by)),
            Exp::LocalComatch(e) => Exp::LocalComatch(e.shift_in_range(range, by)),
            Exp::Hole(e) => Exp::Hole(e.shift_in_range(range, by)),
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
    pub inferred_type: Option<Rc<Exp>>,
}

impl HasSpan for Variable {
    fn span(&self) -> Option<Span> {
        self.span
    }
}

impl From<Variable> for Exp {
    fn from(val: Variable) -> Self {
        Exp::Variable(val)
    }
}

impl Shift for Variable {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let Variable { span, idx, name, .. } = self;
        Variable {
            span: *span,
            idx: idx.shift_in_range(range, by),
            name: name.clone(),
            inferred_type: None,
        }
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
pub struct TypCtor {
    /// Source code location
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub span: Option<Span>,
    /// Name of the type constructor
    pub name: Ident,
    /// Arguments to the type constructor
    pub args: Args,
}

impl TypCtor {
    pub fn to_exp(&self) -> Exp {
        Exp::TypCtor(self.clone())
    }

    /// A type application is simple if the list of arguments is empty.
    pub fn is_simple(&self) -> bool {
        self.args.is_empty()
    }
}

impl HasSpan for TypCtor {
    fn span(&self) -> Option<Span> {
        self.span
    }
}

impl From<TypCtor> for Exp {
    fn from(val: TypCtor) -> Self {
        Exp::TypCtor(val)
    }
}

impl Shift for TypCtor {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let TypCtor { span, name, args } = self;
        TypCtor { span: *span, name: name.clone(), args: args.shift_in_range(range, by) }
    }
}

// Call
//
//

/// A Call invokes a constructor, a codefinition or a toplevel let-bound definition.
/// Examples: `Zero`, `Cons(True, Nil)`, `minimum(x,y)`
#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Call {
    /// Source code location
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub span: Option<Span>,
    /// The name of the call.
    /// The `f` in `f(e1...en)`
    pub name: Ident,
    /// The arguments to the call.
    /// The `(e1...en)` in `f(e1...en)`
    pub args: Args,
    /// The inferred result type of the call.
    /// This type is annotated during elaboration.
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub inferred_type: Option<Rc<Exp>>,
}

impl HasSpan for Call {
    fn span(&self) -> Option<Span> {
        self.span
    }
}

impl From<Call> for Exp {
    fn from(val: Call) -> Self {
        Exp::Call(val)
    }
}

impl Shift for Call {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let Call { span, name, args, .. } = self;
        Call {
            span: *span,
            name: name.clone(),
            args: args.shift_in_range(range, by),
            inferred_type: None,
        }
    }
}

// DotCall
//
//

/// A DotCall is either a destructor or a definition, applied to a destructee
/// Examples: `e.head` `xs.append(ys)`
#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct DotCall {
    /// Source code location
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub span: Option<Span>,
    /// The expression to which the dotcall is applied.
    /// The `e` in `e.f(e1...en)`
    pub exp: Rc<Exp>,
    /// The name of the dotcall.
    /// The `f` in `e.f(e1...en)`
    pub name: Ident,
    /// The arguments of the dotcall.
    /// The `(e1...en)` in `e.f(e1...en)`
    pub args: Args,
    /// The inferred result type of the dotcall.
    /// This type is annotated during elaboration.
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub inferred_type: Option<Rc<Exp>>,
}

impl HasSpan for DotCall {
    fn span(&self) -> Option<Span> {
        self.span
    }
}

impl From<DotCall> for Exp {
    fn from(val: DotCall) -> Self {
        Exp::DotCall(val)
    }
}

impl Shift for DotCall {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let DotCall { span, exp, name, args, .. } = self;
        DotCall {
            span: *span,
            exp: exp.shift_in_range(range.clone(), by),
            name: name.clone(),
            args: args.shift_in_range(range, by),
            inferred_type: None,
        }
    }
}

// Anno
//
//

/// Type annotated term `e : t`
#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Anno {
    /// Source code location
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub span: Option<Span>,
    /// The annotated term, i.e. `e` in `e : t`
    pub exp: Rc<Exp>,
    /// The annotated type, i.e. `t` in `e : t`
    pub typ: Rc<Exp>,
    /// The annotated type written by the user might not
    /// be in normal form. After elaboration we therefore
    /// annotate the normalized type.
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub normalized_type: Option<Rc<Exp>>,
}

impl HasSpan for Anno {
    fn span(&self) -> Option<Span> {
        self.span
    }
}

impl From<Anno> for Exp {
    fn from(val: Anno) -> Self {
        Exp::Anno(val)
    }
}

impl Shift for Anno {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let Anno { span, exp, typ, .. } = self;
        Anno {
            span: *span,
            exp: exp.shift_in_range(range.clone(), by),
            typ: typ.shift_in_range(range, by),
            normalized_type: None,
        }
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

impl From<TypeUniv> for Exp {
    fn from(val: TypeUniv) -> Self {
        Exp::TypeUniv(val)
    }
}

impl Shift for TypeUniv {
    fn shift_in_range<R: ShiftRange>(&self, _range: R, _by: (isize, isize)) -> Self {
        self.clone()
    }
}

// LocalMatch
//
//

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct LocalMatch {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub span: Option<Span>,
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub ctx: Option<TypeCtx>,
    pub name: Label,
    pub on_exp: Rc<Exp>,
    pub motive: Option<Motive>,
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub ret_typ: Option<Rc<Exp>>,
    pub body: Match,
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub inferred_type: Option<TypCtor>,
}

impl HasSpan for LocalMatch {
    fn span(&self) -> Option<Span> {
        self.span
    }
}

impl From<LocalMatch> for Exp {
    fn from(val: LocalMatch) -> Self {
        Exp::LocalMatch(val)
    }
}

impl Shift for LocalMatch {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let LocalMatch { span, name, on_exp, motive, body, .. } = self;
        LocalMatch {
            span: *span,
            ctx: None,
            name: name.clone(),
            on_exp: on_exp.shift_in_range(range.clone(), by),
            motive: motive.shift_in_range(range.clone(), by),
            ret_typ: None,
            body: body.shift_in_range(range, by),
            inferred_type: None,
        }
    }
}

// LocalComatch
//
//

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct LocalComatch {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub span: Option<Span>,
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub ctx: Option<TypeCtx>,
    pub name: Label,
    pub is_lambda_sugar: bool,
    pub body: Match,
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub inferred_type: Option<TypCtor>,
}

impl HasSpan for LocalComatch {
    fn span(&self) -> Option<Span> {
        self.span
    }
}

impl From<LocalComatch> for Exp {
    fn from(val: LocalComatch) -> Self {
        Exp::LocalComatch(val)
    }
}

impl Shift for LocalComatch {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let LocalComatch { span, name, is_lambda_sugar, body, .. } = self;
        LocalComatch {
            span: *span,
            ctx: None,
            name: name.clone(),
            is_lambda_sugar: *is_lambda_sugar,
            body: body.shift_in_range(range, by),
            inferred_type: None,
        }
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
    pub inferred_type: Option<Rc<Exp>>,
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

impl From<Hole> for Exp {
    fn from(val: Hole) -> Self {
        Exp::Hole(val)
    }
}

impl Shift for Hole {
    fn shift_in_range<R: ShiftRange>(&self, _range: R, _by: (isize, isize)) -> Self {
        let Hole { span, .. } = self;
        Hole { span: *span, inferred_type: None, inferred_ctx: None }
    }
}

// Other
//
//

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Match {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub span: Option<Span>,
    pub cases: Vec<Case>,
    pub omit_absurd: bool,
}

impl Shift for Match {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let Match { span, cases, omit_absurd } = self;
        Match { span: *span, cases: cases.shift_in_range(range, by), omit_absurd: *omit_absurd }
    }
}

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Case {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub span: Option<Span>,
    pub name: Ident,
    pub params: TelescopeInst,
    /// Body being `None` represents an absurd pattern
    pub body: Option<Rc<Exp>>,
}

impl Shift for Case {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let Case { span, name, params, body } = self;
        Case {
            span: *span,
            name: name.clone(),
            params: params.clone(),
            body: body.shift_in_range(range.shift(1), by),
        }
    }
}

/// Instantiation of a previously declared telescope
#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct TelescopeInst {
    pub params: Vec<ParamInst>,
}

impl TelescopeInst {
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
pub struct ParamInst {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub span: Option<Span>,
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub info: Option<Rc<Exp>>,
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub name: Ident,
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub typ: Option<Rc<Exp>>,
}

/// A list of arguments
/// In dependent type theory, this concept is usually called a "substitution" but that name would be confusing in this implementation
/// because it conflicts with the operation of substitution (i.e. substituting a terms for a variable in another term).
/// In particular, while we often substitute argument lists for telescopes, this is not always the case.
/// Substitutions in the sense of argument lists are a special case of a more general concept of context morphisms.
/// Unifiers are another example of context morphisms and applying a unifier to an expression mean substituting various terms,
/// which are not necessarily part of a single argument list.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Args {
    pub args: Vec<Rc<Exp>>,
}

impl Args {
    pub fn len(&self) -> usize {
        self.args.len()
    }

    pub fn is_empty(&self) -> bool {
        self.args.is_empty()
    }
}

impl Shift for Args {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        Self { args: self.args.shift_in_range(range, by) }
    }
}

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Motive {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub span: Option<Span>,
    pub param: ParamInst,
    pub ret_typ: Rc<Exp>,
}

impl Shift for Motive {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let Motive { span, param, ret_typ } = self;

        Motive {
            span: *span,
            param: param.clone(),
            ret_typ: ret_typ.shift_in_range(range.shift(1), by),
        }
    }
}
