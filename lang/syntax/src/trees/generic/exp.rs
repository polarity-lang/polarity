use std::fmt;
use std::hash::Hash;
use std::rc::Rc;

use codespan::Span;
use derivative::Derivative;

use crate::common::*;
use crate::ctx::values::TypeCtx;
use crate::ctx::{BindContext, LevelCtx};

use super::traits::Occurs;
use super::{ForgetTST, HasTypeInfo};

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

impl HasTypeInfo for Exp {
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

impl ForgetTST for Exp {
    fn forget_tst(&self) -> Self {
        match self {
            Exp::Variable(e) => Exp::Variable(e.forget_tst()),
            Exp::TypCtor(e) => Exp::TypCtor(e.forget_tst()),
            Exp::Call(e) => Exp::Call(e.forget_tst()),
            Exp::DotCall(e) => Exp::DotCall(e.forget_tst()),
            Exp::Anno(e) => Exp::Anno(e.forget_tst()),
            Exp::TypeUniv(e) => Exp::TypeUniv(e.forget_tst()),
            Exp::LocalMatch(e) => Exp::LocalMatch(e.forget_tst()),
            Exp::LocalComatch(e) => Exp::LocalComatch(e.forget_tst()),
            Exp::Hole(e) => Exp::Hole(e.forget_tst()),
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

impl ForgetTST for Variable {
    fn forget_tst(&self) -> Self {
        let Variable { span, idx, name, .. } = self;
        Variable { span: *span, idx: *idx, name: name.clone(), inferred_type: None }
    }
}

impl HasTypeInfo for Variable {
    fn typ(&self) -> Option<Rc<Exp>> {
        self.inferred_type.clone()
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

impl ForgetTST for TypCtor {
    fn forget_tst(&self) -> Self {
        let TypCtor { span, name, args } = self;
        TypCtor { span: *span, name: name.clone(), args: args.forget_tst() }
    }
}

impl HasTypeInfo for TypCtor {
    fn typ(&self) -> Option<Rc<Exp>> {
        Some(Rc::new(TypeUniv::new().into()))
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

impl Occurs for Call {
    fn occurs(&self, ctx: &mut LevelCtx, lvl: Lvl) -> bool {
        let Call { args, .. } = self;
        args.args.iter().any(|arg| arg.occurs(ctx, lvl))
    }
}

impl ForgetTST for Call {
    fn forget_tst(&self) -> Self {
        let Call { span, name, args, .. } = self;
        Call { span: *span, name: name.clone(), args: args.forget_tst(), inferred_type: None }
    }
}

impl HasTypeInfo for Call {
    fn typ(&self) -> Option<Rc<Exp>> {
        self.inferred_type.clone()
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

impl Occurs for DotCall {
    fn occurs(&self, ctx: &mut LevelCtx, lvl: Lvl) -> bool {
        let DotCall { exp, args, .. } = self;
        exp.occurs(ctx, lvl) || args.args.iter().any(|arg| arg.occurs(ctx, lvl))
    }
}

impl ForgetTST for DotCall {
    fn forget_tst(&self) -> Self {
        let DotCall { span, exp, name, args, .. } = self;
        DotCall {
            span: *span,
            exp: exp.forget_tst(),
            name: name.clone(),
            args: args.forget_tst(),
            inferred_type: None,
        }
    }
}

impl HasTypeInfo for DotCall {
    fn typ(&self) -> Option<Rc<Exp>> {
        self.inferred_type.clone()
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

impl ForgetTST for Anno {
    fn forget_tst(&self) -> Self {
        let Anno { span, exp, typ, .. } = self;
        Anno { span: *span, exp: exp.forget_tst(), typ: typ.forget_tst(), normalized_type: None }
    }
}

impl HasTypeInfo for Anno {
    fn typ(&self) -> Option<Rc<Exp>> {
        self.normalized_type.clone()
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

impl ForgetTST for TypeUniv {
    fn forget_tst(&self) -> Self {
        let TypeUniv { span } = self;
        TypeUniv { span: *span }
    }
}

impl HasTypeInfo for TypeUniv {
    fn typ(&self) -> Option<Rc<Exp>> {
        Some(Rc::new(TypeUniv::new().into()))
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

impl Occurs for LocalMatch {
    fn occurs(&self, ctx: &mut LevelCtx, lvl: Lvl) -> bool {
        let LocalMatch { on_exp, body, .. } = self;
        on_exp.occurs(ctx, lvl) || body.occurs(ctx, lvl)
    }
}

impl ForgetTST for LocalMatch {
    fn forget_tst(&self) -> Self {
        let LocalMatch { span, ctx: _, name, on_exp, motive, ret_typ, body, .. } = self;
        LocalMatch {
            span: *span,
            ctx: None,
            name: name.clone(),
            on_exp: on_exp.forget_tst(),
            motive: motive.as_ref().map(|x| x.forget_tst()),
            ret_typ: ret_typ.as_ref().map(|x| x.forget_tst()),
            body: body.forget_tst(),
            inferred_type: None,
        }
    }
}

impl HasTypeInfo for LocalMatch {
    fn typ(&self) -> Option<Rc<Exp>> {
        self.inferred_type.clone().map(|x| Rc::new(x.into()))
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

impl Occurs for LocalComatch {
    fn occurs(&self, ctx: &mut LevelCtx, lvl: Lvl) -> bool {
        let LocalComatch { body, .. } = self;
        body.occurs(ctx, lvl)
    }
}

impl ForgetTST for LocalComatch {
    fn forget_tst(&self) -> Self {
        let LocalComatch { span, ctx: _, name, is_lambda_sugar, body, .. } = self;

        LocalComatch {
            span: *span,
            ctx: None,
            name: name.clone(),
            is_lambda_sugar: *is_lambda_sugar,
            body: body.forget_tst(),
            inferred_type: None,
        }
    }
}

impl HasTypeInfo for LocalComatch {
    fn typ(&self) -> Option<Rc<Exp>> {
        self.inferred_type.clone().map(|x| Rc::new(x.into()))
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

impl Hole {
    pub fn new() -> Hole {
        Hole { span: None, inferred_type: None, inferred_ctx: None }
    }
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

impl Default for Hole {
    fn default() -> Self {
        Self::new()
    }
}

impl Shift for Hole {
    fn shift_in_range<R: ShiftRange>(&self, _range: R, _by: (isize, isize)) -> Self {
        let Hole { span, .. } = self;
        Hole { span: *span, inferred_type: None, inferred_ctx: None }
    }
}

impl Occurs for Hole {
    fn occurs(&self, _ctx: &mut LevelCtx, _lvl: Lvl) -> bool {
        false
    }
}

impl ForgetTST for Hole {
    fn forget_tst(&self) -> Self {
        let Hole { span, .. } = self;
        Hole { span: *span, inferred_type: None, inferred_ctx: None }
    }
}

impl HasTypeInfo for Hole {
    fn typ(&self) -> Option<Rc<Exp>> {
        self.inferred_type.clone()
    }
}

// Match
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

impl Occurs for Match {
    fn occurs(&self, ctx: &mut LevelCtx, lvl: Lvl) -> bool {
        let Match { cases, .. } = self;
        cases.occurs(ctx, lvl)
    }
}

impl ForgetTST for Match {
    fn forget_tst(&self) -> Self {
        let Match { span, cases, omit_absurd } = self;

        Match { span: *span, cases: cases.forget_tst(), omit_absurd: *omit_absurd }
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

impl ForgetTST for Case {
    fn forget_tst(&self) -> Self {
        let Case { span, name, params, body } = self;

        Case {
            span: *span,
            name: name.clone(),
            params: params.forget_tst(),
            body: body.as_ref().map(|x| x.forget_tst()),
        }
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

impl ForgetTST for TelescopeInst {
    fn forget_tst(&self) -> Self {
        let TelescopeInst { params } = self;

        TelescopeInst { params: params.forget_tst() }
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

impl ForgetTST for ParamInst {
    fn forget_tst(&self) -> Self {
        let ParamInst { span, name, typ, .. } = self;

        ParamInst {
            span: *span,
            info: None,
            name: name.clone(),
            typ: typ.as_ref().map(|x| x.forget_tst()),
        }
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

impl ForgetTST for Args {
    fn forget_tst(&self) -> Self {
        Args { args: self.args.forget_tst() }
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

impl ForgetTST for Motive {
    fn forget_tst(&self) -> Self {
        let Motive { span, param, ret_typ } = self;

        Motive { span: *span, param: param.forget_tst(), ret_typ: ret_typ.forget_tst() }
    }
}
