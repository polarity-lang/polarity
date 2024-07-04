use std::fmt;
use std::hash::Hash;
use std::rc::Rc;

use codespan::Span;
use derivative::Derivative;

use crate::ctx::values::TypeCtx;
use crate::ctx::{BindContext, LevelCtx};

use super::subst::{Substitutable, Substitution};
use super::traits::HasSpan;
use super::traits::Occurs;
use super::{ident::*, Shift, ShiftRange, ShiftRangeExt};
use super::{HasType, Leveled};

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

// Exp
//
//

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

impl Exp {
    pub fn to_typctor(self) -> Option<TypCtor> {
        match self {
            Exp::TypCtor(e) => Some(e),
            _ => None,
        }
    }
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

impl Occurs for Exp {
    fn occurs(&self, ctx: &mut LevelCtx, lvl: Lvl) -> bool {
        match self {
            Exp::Variable(e) => e.occurs(ctx, lvl),
            Exp::TypCtor(e) => e.occurs(ctx, lvl),
            Exp::Call(e) => e.occurs(ctx, lvl),
            Exp::DotCall(e) => e.occurs(ctx, lvl),
            Exp::Anno(e) => e.occurs(ctx, lvl),
            Exp::TypeUniv(e) => e.occurs(ctx, lvl),
            Exp::LocalMatch(e) => e.occurs(ctx, lvl),
            Exp::LocalComatch(e) => e.occurs(ctx, lvl),
            Exp::Hole(e) => e.occurs(ctx, lvl),
        }
    }
}

impl HasType for Exp {
    fn typ(&self) -> Option<Rc<Exp>> {
        match self {
            Exp::Variable(e) => e.typ(),
            Exp::TypCtor(e) => e.typ(),
            Exp::Call(e) => e.typ(),
            Exp::DotCall(e) => e.typ(),
            Exp::Anno(e) => e.typ(),
            Exp::TypeUniv(e) => e.typ(),
            Exp::LocalMatch(e) => e.typ(),
            Exp::LocalComatch(e) => e.typ(),
            Exp::Hole(e) => e.typ(),
        }
    }
}

impl Substitutable for Rc<Exp> {
    type Result = Rc<Exp>;
    fn subst<S: Substitution>(&self, ctx: &mut LevelCtx, by: &S) -> Self {
        match &**self {
            Exp::Variable(e) => e.subst(ctx, by),
            Exp::TypCtor(e) => Rc::new(e.subst(ctx, by).into()),
            Exp::Call(e) => Rc::new(e.subst(ctx, by).into()),
            Exp::DotCall(e) => Rc::new(e.subst(ctx, by).into()),
            Exp::Anno(e) => Rc::new(e.subst(ctx, by).into()),
            Exp::TypeUniv(e) => Rc::new(e.subst(ctx, by).into()),
            Exp::LocalMatch(e) => Rc::new(e.subst(ctx, by).into()),
            Exp::LocalComatch(e) => Rc::new(e.subst(ctx, by).into()),
            Exp::Hole(e) => Rc::new(e.subst(ctx, by).into()),
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

impl Occurs for Variable {
    fn occurs(&self, ctx: &mut LevelCtx, lvl: Lvl) -> bool {
        let Variable { idx, .. } = self;
        ctx.idx_to_lvl(*idx) == lvl
    }
}

impl HasType for Variable {
    fn typ(&self) -> Option<Rc<Exp>> {
        self.inferred_type.clone()
    }
}

impl Substitutable for Variable {
    type Result = Rc<Exp>;
    fn subst<S: Substitution>(&self, ctx: &mut LevelCtx, by: &S) -> Self::Result {
        let Variable { span, idx, name, .. } = self;
        match by.get_subst(ctx, ctx.idx_to_lvl(*idx)) {
            Some(exp) => exp,
            None => Rc::new(Exp::Variable(Variable {
                span: *span,
                idx: *idx,
                name: name.clone(),
                inferred_type: None,
            })),
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

impl Occurs for TypCtor {
    fn occurs(&self, ctx: &mut LevelCtx, lvl: Lvl) -> bool {
        let TypCtor { args, .. } = self;
        args.args.iter().any(|arg| arg.occurs(ctx, lvl))
    }
}

impl HasType for TypCtor {
    fn typ(&self) -> Option<Rc<Exp>> {
        Some(Rc::new(TypeUniv::new().into()))
    }
}

impl Substitutable for TypCtor {
    type Result = TypCtor;
    fn subst<S: Substitution>(&self, ctx: &mut LevelCtx, by: &S) -> Self {
        let TypCtor { span, name, args } = self;
        TypCtor { span: *span, name: name.clone(), args: args.subst(ctx, by) }
    }
}

// Call
//
//

/// A Call expression can be one of three different kinds:
/// - A constructor introduced by a data type declaration
/// - A codefinition introduced at the toplevel
/// - A LetBound definition introduced at the toplevel
#[derive(Debug, Clone, Copy, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub enum CallKind {
    Constructor,
    Codefinition,
    LetBound,
}

/// A Call invokes a constructor, a codefinition or a toplevel let-bound definition.
/// Examples: `Zero`, `Cons(True, Nil)`, `minimum(x,y)`
#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Call {
    /// Source code location
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub span: Option<Span>,
    /// Whether the call is a constructor, codefinition or let bound definition.
    pub kind: CallKind,
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
        let Call { span, name, args, kind, .. } = self;
        Call {
            span: *span,
            kind: *kind,
            name: name.clone(),
            args: args.shift_in_range(range, by),
            inferred_type: None,
        }
    }
}

impl Occurs for Call {
    fn occurs(&self, ctx: &mut LevelCtx, lvl: Lvl) -> bool {
        let Call { args, .. } = self;
        args.args.iter().any(|arg| arg.occurs(ctx, lvl))
    }
}

impl HasType for Call {
    fn typ(&self) -> Option<Rc<Exp>> {
        self.inferred_type.clone()
    }
}

impl Substitutable for Call {
    type Result = Call;
    fn subst<S: Substitution>(&self, ctx: &mut LevelCtx, by: &S) -> Self::Result {
        let Call { span, name, args, kind, .. } = self;
        Call {
            span: *span,
            kind: *kind,
            name: name.clone(),
            args: args.subst(ctx, by),
            inferred_type: None,
        }
    }
}

// DotCall
//
//

/// A DotCall expression can be one of two different kinds:
/// - A destructor introduced by a codata type declaration
/// - A definition introduced at the toplevel
#[derive(Debug, Clone, Copy, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub enum DotCallKind {
    Destructor,
    Definition,
}

/// A DotCall is either a destructor or a definition, applied to a destructee
/// Examples: `e.head` `xs.append(ys)`
#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct DotCall {
    /// Source code location
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub span: Option<Span>,
    /// Whether the dotcall is a destructor or codefinition.
    pub kind: DotCallKind,
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
        let DotCall { span, kind, exp, name, args, .. } = self;
        DotCall {
            span: *span,
            kind: *kind,
            exp: exp.shift_in_range(range.clone(), by),
            name: name.clone(),
            args: args.shift_in_range(range, by),
            inferred_type: None,
        }
    }
}

impl Occurs for DotCall {
    fn occurs(&self, ctx: &mut LevelCtx, lvl: Lvl) -> bool {
        let DotCall { exp, args, .. } = self;
        exp.occurs(ctx, lvl) || args.args.iter().any(|arg| arg.occurs(ctx, lvl))
    }
}

impl HasType for DotCall {
    fn typ(&self) -> Option<Rc<Exp>> {
        self.inferred_type.clone()
    }
}

impl Substitutable for DotCall {
    type Result = DotCall;
    fn subst<S: Substitution>(&self, ctx: &mut LevelCtx, by: &S) -> Self::Result {
        let DotCall { span, kind, exp, name, args, .. } = self;
        DotCall {
            span: *span,
            kind: *kind,
            exp: exp.subst(ctx, by),
            name: name.clone(),
            args: args.subst(ctx, by),
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

impl Occurs for Anno {
    fn occurs(&self, ctx: &mut LevelCtx, lvl: Lvl) -> bool {
        let Anno { exp, typ, .. } = self;
        exp.occurs(ctx, lvl) || typ.occurs(ctx, lvl)
    }
}

impl HasType for Anno {
    fn typ(&self) -> Option<Rc<Exp>> {
        self.normalized_type.clone()
    }
}

impl Substitutable for Anno {
    type Result = Anno;

    fn subst<S: Substitution>(&self, ctx: &mut LevelCtx, by: &S) -> Self::Result {
        let Anno { span, exp, typ, .. } = self;
        Anno {
            span: *span,
            exp: exp.subst(ctx, by),
            typ: typ.subst(ctx, by),
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

impl TypeUniv {
    pub fn new() -> TypeUniv {
        TypeUniv { span: None }
    }
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

impl Default for TypeUniv {
    fn default() -> Self {
        Self::new()
    }
}

impl Shift for TypeUniv {
    fn shift_in_range<R: ShiftRange>(&self, _range: R, _by: (isize, isize)) -> Self {
        self.clone()
    }
}

impl Occurs for TypeUniv {
    fn occurs(&self, _ctx: &mut LevelCtx, _lvl: Lvl) -> bool {
        false
    }
}

impl HasType for TypeUniv {
    fn typ(&self) -> Option<Rc<Exp>> {
        Some(Rc::new(TypeUniv::new().into()))
    }
}

impl Substitutable for TypeUniv {
    type Result = TypeUniv;

    fn subst<S: Substitution>(&self, _ctx: &mut LevelCtx, _by: &S) -> Self::Result {
        let TypeUniv { span } = self;
        TypeUniv { span: *span }
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
    pub cases: Vec<Case>,
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
        let LocalMatch { span, name, on_exp, motive, cases, .. } = self;
        LocalMatch {
            span: *span,
            ctx: None,
            name: name.clone(),
            on_exp: on_exp.shift_in_range(range.clone(), by),
            motive: motive.shift_in_range(range.clone(), by),
            ret_typ: None,
            cases: cases.shift_in_range(range, by),
            inferred_type: None,
        }
    }
}

impl Occurs for LocalMatch {
    fn occurs(&self, ctx: &mut LevelCtx, lvl: Lvl) -> bool {
        let LocalMatch { on_exp, cases, .. } = self;
        on_exp.occurs(ctx, lvl) || cases.occurs(ctx, lvl)
    }
}

impl HasType for LocalMatch {
    fn typ(&self) -> Option<Rc<Exp>> {
        self.inferred_type.clone().map(|x| Rc::new(x.into()))
    }
}

impl Substitutable for LocalMatch {
    type Result = LocalMatch;
    fn subst<S: Substitution>(&self, ctx: &mut LevelCtx, by: &S) -> Self::Result {
        let LocalMatch { span, name, on_exp, motive, ret_typ, cases, .. } = self;
        LocalMatch {
            span: *span,
            ctx: None,
            name: name.clone(),
            on_exp: on_exp.subst(ctx, by),
            motive: motive.subst(ctx, by),
            ret_typ: ret_typ.subst(ctx, by),
            cases: cases.iter().map(|case| case.subst(ctx, by)).collect(),
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
    pub cases: Vec<Case>,
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
        let LocalComatch { span, name, is_lambda_sugar, cases, .. } = self;
        LocalComatch {
            span: *span,
            ctx: None,
            name: name.clone(),
            is_lambda_sugar: *is_lambda_sugar,
            cases: cases.shift_in_range(range, by),
            inferred_type: None,
        }
    }
}

impl Occurs for LocalComatch {
    fn occurs(&self, ctx: &mut LevelCtx, lvl: Lvl) -> bool {
        let LocalComatch { cases, .. } = self;
        cases.occurs(ctx, lvl)
    }
}

impl HasType for LocalComatch {
    fn typ(&self) -> Option<Rc<Exp>> {
        self.inferred_type.clone().map(|x| Rc::new(x.into()))
    }
}

impl Substitutable for LocalComatch {
    type Result = LocalComatch;

    fn subst<S: Substitution>(&self, ctx: &mut LevelCtx, by: &S) -> Self::Result {
        let LocalComatch { span, name, is_lambda_sugar, cases, .. } = self;
        LocalComatch {
            span: *span,
            ctx: None,
            name: name.clone(),
            is_lambda_sugar: *is_lambda_sugar,
            cases: cases.iter().map(|case| case.subst(ctx, by)).collect(),
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
    /// The metavariable that we want to solve for that hole
    pub metavar: MetaVar,
    /// The inferred type of the hole annotated during elaboration.
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub inferred_type: Option<Rc<Exp>>,
    /// The type context in which the hole occurs.
    /// This context is annotated during elaboration.
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub inferred_ctx: Option<TypeCtx>,
    /// When a hole is lowered, we apply it to all variables available in the
    /// context, since intuitively, a hole stands for an unknown term which can use
    /// any of these variables.
    /// Some other implementations use a functional application to all variables instead,
    /// but since we do not have a function type we have to use an explicit substitution.
    /// Since our system uses 2-level De-Bruijn indices, the explicit substitution id_Г
    /// is a nested vector.
    ///
    /// Example:
    /// [x, y][z][v, w] |- ?[x, y][z][v,w]
    pub args: Vec<Vec<Rc<Exp>>>,
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
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let Hole { span, metavar, args, .. } = self;
        let new_args = args.shift_in_range(range, by);
        Hole {
            span: *span,
            metavar: *metavar,
            inferred_type: None,
            inferred_ctx: None,
            args: new_args,
        }
    }
}

impl Occurs for Hole {
    fn occurs(&self, _ctx: &mut LevelCtx, _lvl: Lvl) -> bool {
        false
    }
}

impl HasType for Hole {
    fn typ(&self) -> Option<Rc<Exp>> {
        self.inferred_type.clone()
    }
}

impl Substitutable for Hole {
    type Result = Hole;

    fn subst<S: Substitution>(&self, ctx: &mut LevelCtx, by: &S) -> Self::Result {
        let Hole { span, metavar, args, .. } = self;
        Hole {
            span: *span,
            metavar: *metavar,
            inferred_type: None,
            inferred_ctx: None,
            args: args.subst(ctx, by),
        }
    }
}

// Case
//
//

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

impl Occurs for Case {
    fn occurs(&self, ctx: &mut LevelCtx, lvl: Lvl) -> bool {
        let Case { params, body, .. } = self;
        ctx.bind_iter(params.params.iter().map(|_| ()), |ctx| body.occurs(ctx, lvl))
    }
}

impl Substitutable for Case {
    type Result = Case;
    fn subst<S: Substitution>(&self, ctx: &mut LevelCtx, by: &S) -> Self {
        let Case { span, name, params, body } = self;
        ctx.bind_iter(params.params.iter(), |ctx| Case {
            span: *span,
            name: name.clone(),
            params: params.clone(),
            body: body.as_ref().map(|body| body.subst(ctx, &by.shift((1, 0)))),
        })
    }
}

// Telescope Inst
//
//

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

// ParamInst
//
//

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

impl Named for ParamInst {
    fn name(&self) -> &Ident {
        &self.name
    }
}

// Args
//
//

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

impl Substitutable for Args {
    type Result = Args;
    fn subst<S: Substitution>(&self, ctx: &mut LevelCtx, by: &S) -> Self {
        Args { args: self.args.subst(ctx, by) }
    }
}

// Motive
//
//

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

impl Substitutable for Motive {
    type Result = Motive;
    fn subst<S: Substitution>(&self, ctx: &mut LevelCtx, by: &S) -> Self {
        let Motive { span, param, ret_typ } = self;

        Motive {
            span: *span,
            param: param.clone(),
            ret_typ: ctx.bind_single((), |ctx| ret_typ.subst(ctx, &by.shift((1, 0)))),
        }
    }
}
