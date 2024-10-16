use std::fmt;
use std::hash::Hash;

use codespan::Span;
use derivative::Derivative;
use pretty::DocAllocator;
use printer::theme::ThemeExt;
use printer::tokens::{
    ABSURD, ARROW, AS, COLON, COLONEQ, COMATCH, COMMA, DOT, FAT_ARROW, MATCH, QUESTION_MARK, TYPE,
    UNDERSCORE,
};
use printer::util::{BackslashExt, BracesExt, IsNilExt};
use printer::{Alloc, Builder, Precedence, Print, PrintCfg};

use crate::ctx::values::TypeCtx;
use crate::ctx::{BindContext, LevelCtx};
use crate::named::Named;
use crate::{SubstUnderCtx, Zonk, ZonkError};

use super::subst::{Substitutable, Substitution};
use super::traits::HasSpan;
use super::traits::Occurs;
use super::HasType;
use super::{ident::*, Shift, ShiftRange, ShiftRangeExt};

// Prints "{ }"
pub fn empty_braces<'a>(alloc: &'a Alloc<'a>) -> Builder<'a> {
    alloc.space().braces_anno()
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

// Arg
//
//

/// Arguments in an argument list can either be unnamed or named.
/// Example for named arguments: `f(x := 1, y := 2)`
/// Example for unnamed arguments: `f(1, 2)`
#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub enum Arg {
    UnnamedArg(Box<Exp>),
    NamedArg(Ident, Box<Exp>),
    InsertedImplicitArg(Hole),
}

impl Arg {
    pub fn is_inserted_implicit(&self) -> bool {
        matches!(self, Arg::InsertedImplicitArg(_))
    }

    pub fn exp(&self) -> Box<Exp> {
        match self {
            Arg::UnnamedArg(e) => e.clone(),
            Arg::NamedArg(_, e) => e.clone(),
            Arg::InsertedImplicitArg(hole) => Box::new(hole.clone().into()),
        }
    }
}

impl HasSpan for Arg {
    fn span(&self) -> Option<Span> {
        match self {
            Arg::UnnamedArg(e) => e.span(),
            Arg::NamedArg(_, e) => e.span(),
            Arg::InsertedImplicitArg(hole) => hole.span(),
        }
    }
}

impl Shift for Arg {
    fn shift_in_range<R: ShiftRange>(&mut self, range: &R, by: (isize, isize)) {
        match self {
            Arg::UnnamedArg(e) => e.shift_in_range(range, by),
            Arg::NamedArg(_, e) => e.shift_in_range(range, by),
            Arg::InsertedImplicitArg(hole) => hole.shift_in_range(range, by),
        }
    }
}

impl Occurs for Arg {
    fn occurs(&self, ctx: &mut LevelCtx, lvl: Lvl) -> bool {
        match self {
            Arg::UnnamedArg(e) => e.occurs(ctx, lvl),
            Arg::NamedArg(_, e) => e.occurs(ctx, lvl),
            Arg::InsertedImplicitArg(hole) => hole.occurs(ctx, lvl),
        }
    }
}

impl HasType for Arg {
    fn typ(&self) -> Option<Box<Exp>> {
        match self {
            Arg::UnnamedArg(e) => e.typ(),
            Arg::NamedArg(_, e) => e.typ(),
            Arg::InsertedImplicitArg(hole) => hole.typ(),
        }
    }
}

impl Substitutable for Arg {
    type Result = Arg;
    fn subst<S: Substitution>(&self, ctx: &mut LevelCtx, by: &S) -> Self::Result {
        match self {
            Arg::UnnamedArg(e) => Arg::UnnamedArg(e.subst(ctx, by)),
            Arg::NamedArg(i, e) => Arg::NamedArg(i.clone(), e.subst(ctx, by)),
            Arg::InsertedImplicitArg(hole) => Arg::InsertedImplicitArg(hole.subst(ctx, by)),
        }
    }
}

impl Print for Arg {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        match self {
            Arg::UnnamedArg(e) => e.print(cfg, alloc),
            Arg::NamedArg(i, e) => alloc.text(&i.id).append(COLONEQ).append(e.print(cfg, alloc)),
            Arg::InsertedImplicitArg(_) => {
                panic!("Inserted implicit arguments should not be printed")
            }
        }
    }
}

impl Zonk for Arg {
    fn zonk(
        &mut self,
        meta_vars: &crate::HashMap<MetaVar, crate::MetaVarState>,
    ) -> Result<(), ZonkError> {
        match self {
            Arg::UnnamedArg(e) => e.zonk(meta_vars),
            Arg::NamedArg(_, e) => e.zonk(meta_vars),
            Arg::InsertedImplicitArg(hole) => hole.zonk(meta_vars),
        }
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
    fn shift_in_range<R: ShiftRange>(&mut self, range: &R, by: (isize, isize)) {
        match self {
            Exp::Variable(e) => e.shift_in_range(range, by),
            Exp::TypCtor(e) => e.shift_in_range(range, by),
            Exp::Call(e) => e.shift_in_range(range, by),
            Exp::DotCall(e) => e.shift_in_range(range, by),
            Exp::Anno(e) => e.shift_in_range(range, by),
            Exp::TypeUniv(e) => e.shift_in_range(range, by),
            Exp::LocalMatch(e) => e.shift_in_range(range, by),
            Exp::LocalComatch(e) => e.shift_in_range(range, by),
            Exp::Hole(e) => e.shift_in_range(range, by),
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
    fn typ(&self) -> Option<Box<Exp>> {
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

impl Substitutable for Exp {
    type Result = Exp;
    fn subst<S: Substitution>(&self, ctx: &mut LevelCtx, by: &S) -> Self::Result {
        match self {
            Exp::Variable(e) => *e.subst(ctx, by),
            Exp::TypCtor(e) => e.subst(ctx, by).into(),
            Exp::Call(e) => e.subst(ctx, by).into(),
            Exp::DotCall(e) => e.subst(ctx, by).into(),
            Exp::Anno(e) => e.subst(ctx, by).into(),
            Exp::TypeUniv(e) => e.subst(ctx, by).into(),
            Exp::LocalMatch(e) => e.subst(ctx, by).into(),
            Exp::LocalComatch(e) => e.subst(ctx, by).into(),
            Exp::Hole(e) => e.subst(ctx, by).into(),
        }
    }
}

impl Print for Exp {
    fn print_prec<'a>(
        &'a self,
        cfg: &PrintCfg,
        alloc: &'a Alloc<'a>,
        prec: Precedence,
    ) -> Builder<'a> {
        match self {
            Exp::Variable(e) => e.print_prec(cfg, alloc, prec),
            Exp::TypCtor(e) => e.print_prec(cfg, alloc, prec),
            Exp::Call(e) => e.print_prec(cfg, alloc, prec),
            mut dtor @ Exp::DotCall(DotCall { .. }) => {
                // A series of destructors forms an aligned group
                let mut dtors_group = alloc.nil();
                while let Exp::DotCall(DotCall { exp, name, args, .. }) = &dtor {
                    let psubst = if args.is_empty() { alloc.nil() } else { args.print(cfg, alloc) };
                    if !dtors_group.is_nil() {
                        dtors_group = alloc.line_().append(dtors_group);
                    }
                    dtors_group = alloc
                        .text(DOT)
                        .append(alloc.dtor(&name.id))
                        .append(psubst)
                        .append(dtors_group);
                    dtor = exp;
                }
                dtor.print(cfg, alloc).append(dtors_group.align().group())
            }
            Exp::Anno(e) => e.print_prec(cfg, alloc, prec),
            Exp::TypeUniv(e) => e.print_prec(cfg, alloc, prec),
            Exp::LocalMatch(e) => e.print_prec(cfg, alloc, prec),
            Exp::LocalComatch(e) => e.print_prec(cfg, alloc, prec),
            Exp::Hole(e) => e.print_prec(cfg, alloc, prec),
        }
    }
}

impl Zonk for Exp {
    fn zonk(
        &mut self,
        meta_vars: &crate::HashMap<MetaVar, crate::MetaVarState>,
    ) -> Result<(), ZonkError> {
        match self {
            Exp::Variable(e) => e.zonk(meta_vars),
            Exp::TypCtor(e) => e.zonk(meta_vars),
            Exp::Call(e) => e.zonk(meta_vars),
            Exp::DotCall(e) => e.zonk(meta_vars),
            Exp::Anno(e) => e.zonk(meta_vars),
            Exp::TypeUniv(e) => e.zonk(meta_vars),
            Exp::LocalMatch(e) => e.zonk(meta_vars),
            Exp::LocalComatch(e) => e.zonk(meta_vars),
            Exp::Hole(e) => e.zonk(meta_vars),
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
    pub inferred_type: Option<Box<Exp>>,
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
    fn shift_in_range<R: ShiftRange>(&mut self, range: &R, by: (isize, isize)) {
        self.idx.shift_in_range(range, by);
        self.inferred_type = None;
    }
}

impl Occurs for Variable {
    fn occurs(&self, ctx: &mut LevelCtx, lvl: Lvl) -> bool {
        let Variable { idx, .. } = self;
        ctx.idx_to_lvl(*idx) == lvl
    }
}

impl HasType for Variable {
    fn typ(&self) -> Option<Box<Exp>> {
        self.inferred_type.clone()
    }
}

impl Substitutable for Variable {
    type Result = Box<Exp>;
    fn subst<S: Substitution>(&self, ctx: &mut LevelCtx, by: &S) -> Self::Result {
        let Variable { span, idx, name, .. } = self;
        match by.get_subst(ctx, ctx.idx_to_lvl(*idx)) {
            Some(exp) => exp,
            None => Box::new(Exp::Variable(Variable {
                span: *span,
                idx: *idx,
                name: name.clone(),
                inferred_type: None,
            })),
        }
    }
}

impl Print for Variable {
    fn print_prec<'a>(
        &'a self,
        cfg: &PrintCfg,
        alloc: &'a Alloc<'a>,
        _prec: Precedence,
    ) -> Builder<'a> {
        let Variable { name, idx, .. } = self;
        if cfg.de_bruijn {
            alloc.text(format!("{name}@{idx}"))
        } else if name.id.is_empty() {
            alloc.text(format!("@{idx}"))
        } else {
            alloc.text(&name.id)
        }
    }
}

impl Zonk for Variable {
    fn zonk(
        &mut self,
        meta_vars: &crate::HashMap<MetaVar, crate::MetaVarState>,
    ) -> Result<(), ZonkError> {
        let Variable { span: _, idx: _, name: _, inferred_type } = self;
        inferred_type.zonk(meta_vars)?;
        Ok(())
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
    fn shift_in_range<R: ShiftRange>(&mut self, range: &R, by: (isize, isize)) {
        self.args.shift_in_range(range, by);
    }
}

impl Occurs for TypCtor {
    fn occurs(&self, ctx: &mut LevelCtx, lvl: Lvl) -> bool {
        let TypCtor { args, .. } = self;
        args.args.iter().any(|arg| arg.occurs(ctx, lvl))
    }
}

impl HasType for TypCtor {
    fn typ(&self) -> Option<Box<Exp>> {
        Some(Box::new(TypeUniv::new().into()))
    }
}

impl Substitutable for TypCtor {
    type Result = TypCtor;
    fn subst<S: Substitution>(&self, ctx: &mut LevelCtx, by: &S) -> Self {
        let TypCtor { span, name, args } = self;
        TypCtor { span: *span, name: name.clone(), args: args.subst(ctx, by) }
    }
}

impl Print for TypCtor {
    fn print_prec<'a>(
        &'a self,
        cfg: &PrintCfg,
        alloc: &'a Alloc<'a>,
        prec: Precedence,
    ) -> Builder<'a> {
        let TypCtor { span: _, name, args } = self;
        if name.id == "Fun" && args.len() == 2 && cfg.print_function_sugar {
            let arg = args.args[0].print_prec(cfg, alloc, 1);
            let res = args.args[1].print_prec(cfg, alloc, 0);
            let fun = arg.append(alloc.space()).append(ARROW).append(alloc.space()).append(res);
            if prec == 0 {
                fun
            } else {
                fun.parens()
            }
        } else {
            let psubst = if args.is_empty() { alloc.nil() } else { args.print(cfg, alloc) };
            alloc.typ(&name.id).append(psubst)
        }
    }
}

/// Implement Zonk for TypCtor
impl Zonk for TypCtor {
    fn zonk(
        &mut self,
        meta_vars: &crate::HashMap<MetaVar, crate::MetaVarState>,
    ) -> Result<(), ZonkError> {
        let TypCtor { span: _, name: _, args } = self;
        args.zonk(meta_vars)
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
    pub inferred_type: Option<Box<Exp>>,
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
    fn shift_in_range<R: ShiftRange>(&mut self, range: &R, by: (isize, isize)) {
        self.args.shift_in_range(range, by);
        self.inferred_type = None;
    }
}

impl Occurs for Call {
    fn occurs(&self, ctx: &mut LevelCtx, lvl: Lvl) -> bool {
        let Call { args, .. } = self;
        args.args.iter().any(|arg| arg.occurs(ctx, lvl))
    }
}

impl HasType for Call {
    fn typ(&self) -> Option<Box<Exp>> {
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

impl Print for Call {
    fn print_prec<'a>(
        &'a self,
        cfg: &PrintCfg,
        alloc: &'a Alloc<'a>,
        _prec: Precedence,
    ) -> Builder<'a> {
        let Call { name, args, .. } = self;
        let psubst = if args.is_empty() { alloc.nil() } else { args.print(cfg, alloc) };
        alloc.ctor(&name.id).append(psubst)
    }
}

impl Zonk for Call {
    fn zonk(
        &mut self,
        meta_vars: &crate::HashMap<MetaVar, crate::MetaVarState>,
    ) -> Result<(), ZonkError> {
        let Call { span: _, kind: _, name: _, args, inferred_type } = self;
        args.zonk(meta_vars)?;
        inferred_type.zonk(meta_vars)?;
        Ok(())
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
    pub exp: Box<Exp>,
    /// The name of the dotcall.
    /// The `f` in `e.f(e1...en)`
    pub name: Ident,
    /// The arguments of the dotcall.
    /// The `(e1...en)` in `e.f(e1...en)`
    pub args: Args,
    /// The inferred result type of the dotcall.
    /// This type is annotated during elaboration.
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub inferred_type: Option<Box<Exp>>,
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
    fn shift_in_range<R: ShiftRange>(&mut self, range: &R, by: (isize, isize)) {
        self.exp.shift_in_range(range, by);
        self.args.shift_in_range(range, by);
        self.inferred_type = None;
    }
}

impl Occurs for DotCall {
    fn occurs(&self, ctx: &mut LevelCtx, lvl: Lvl) -> bool {
        let DotCall { exp, args, .. } = self;
        exp.occurs(ctx, lvl) || args.args.iter().any(|arg| arg.occurs(ctx, lvl))
    }
}

impl HasType for DotCall {
    fn typ(&self) -> Option<Box<Exp>> {
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

impl Zonk for DotCall {
    fn zonk(
        &mut self,
        meta_vars: &crate::HashMap<MetaVar, crate::MetaVarState>,
    ) -> Result<(), ZonkError> {
        let DotCall { span: _, kind: _, exp, name: _, args, inferred_type } = self;
        exp.zonk(meta_vars)?;
        args.zonk(meta_vars)?;
        inferred_type.zonk(meta_vars)?;
        Ok(())
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
    pub exp: Box<Exp>,
    /// The annotated type, i.e. `t` in `e : t`
    pub typ: Box<Exp>,
    /// The annotated type written by the user might not
    /// be in normal form. After elaboration we therefore
    /// annotate the normalized type.
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub normalized_type: Option<Box<Exp>>,
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
    fn shift_in_range<R: ShiftRange>(&mut self, range: &R, by: (isize, isize)) {
        self.exp.shift_in_range(range, by);
        self.typ.shift_in_range(range, by);
        self.normalized_type = None;
    }
}

impl Occurs for Anno {
    fn occurs(&self, ctx: &mut LevelCtx, lvl: Lvl) -> bool {
        let Anno { exp, typ, .. } = self;
        exp.occurs(ctx, lvl) || typ.occurs(ctx, lvl)
    }
}

impl HasType for Anno {
    fn typ(&self) -> Option<Box<Exp>> {
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

impl Print for Anno {
    fn print_prec<'a>(
        &'a self,
        cfg: &PrintCfg,
        alloc: &'a Alloc<'a>,
        _prec: Precedence,
    ) -> Builder<'a> {
        let Anno { exp, typ, .. } = self;
        exp.print(cfg, alloc).parens().append(COLON).append(typ.print(cfg, alloc))
    }
}

impl Zonk for Anno {
    fn zonk(
        &mut self,
        meta_vars: &crate::HashMap<MetaVar, crate::MetaVarState>,
    ) -> Result<(), ZonkError> {
        let Anno { span: _, exp, typ, normalized_type } = self;
        exp.zonk(meta_vars)?;
        typ.zonk(meta_vars)?;
        normalized_type.zonk(meta_vars)?;
        Ok(())
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
    fn shift_in_range<R: ShiftRange>(&mut self, _range: &R, _by: (isize, isize)) {}
}

impl Occurs for TypeUniv {
    fn occurs(&self, _ctx: &mut LevelCtx, _lvl: Lvl) -> bool {
        false
    }
}

impl HasType for TypeUniv {
    fn typ(&self) -> Option<Box<Exp>> {
        Some(Box::new(TypeUniv::new().into()))
    }
}

impl Substitutable for TypeUniv {
    type Result = TypeUniv;

    fn subst<S: Substitution>(&self, _ctx: &mut LevelCtx, _by: &S) -> Self::Result {
        let TypeUniv { span } = self;
        TypeUniv { span: *span }
    }
}

impl Print for TypeUniv {
    fn print_prec<'a>(
        &'a self,
        _cfg: &PrintCfg,
        alloc: &'a Alloc<'a>,
        _prec: Precedence,
    ) -> Builder<'a> {
        alloc.keyword(TYPE)
    }
}

/// Implement Zonk for TypeUniv
impl Zonk for TypeUniv {
    fn zonk(
        &mut self,
        _meta_vars: &crate::HashMap<MetaVar, crate::MetaVarState>,
    ) -> Result<(), ZonkError> {
        // TypeUniv has no fields that require zonking
        Ok(())
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
    pub on_exp: Box<Exp>,
    pub motive: Option<Motive>,
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub ret_typ: Option<Box<Exp>>,
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
    fn shift_in_range<R: ShiftRange>(&mut self, range: &R, by: (isize, isize)) {
        self.ctx = None;
        self.on_exp.shift_in_range(range, by);
        self.motive.shift_in_range(range, by);
        self.ret_typ = None;
        self.cases.shift_in_range(range, by);
        self.inferred_type = None;
    }
}

impl Occurs for LocalMatch {
    fn occurs(&self, ctx: &mut LevelCtx, lvl: Lvl) -> bool {
        let LocalMatch { on_exp, cases, .. } = self;
        on_exp.occurs(ctx, lvl) || cases.occurs(ctx, lvl)
    }
}

impl HasType for LocalMatch {
    fn typ(&self) -> Option<Box<Exp>> {
        self.inferred_type.clone().map(|x| Box::new(x.into()))
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

impl Print for LocalMatch {
    fn print_prec<'a>(
        &'a self,
        cfg: &PrintCfg,
        alloc: &'a Alloc<'a>,
        _prec: Precedence,
    ) -> Builder<'a> {
        let LocalMatch { name, on_exp, motive, cases, .. } = self;
        on_exp
            .print(cfg, alloc)
            .append(DOT)
            .append(alloc.keyword(MATCH))
            .append(match &name.user_name {
                Some(name) => alloc.space().append(alloc.dtor(&name.id)),
                None => alloc.nil(),
            })
            .append(motive.as_ref().map(|m| m.print(cfg, alloc)).unwrap_or(alloc.nil()))
            .append(alloc.space())
            .append(print_cases(cases, cfg, alloc))
    }
}

impl Zonk for LocalMatch {
    fn zonk(
        &mut self,
        meta_vars: &crate::HashMap<MetaVar, crate::MetaVarState>,
    ) -> Result<(), ZonkError> {
        let LocalMatch { span: _, ctx: _, name: _, on_exp, motive, ret_typ, cases, inferred_type } =
            self;
        on_exp.zonk(meta_vars)?;
        motive.zonk(meta_vars)?;
        ret_typ.zonk(meta_vars)?;
        inferred_type.zonk(meta_vars)?;
        for case in cases {
            case.zonk(meta_vars)?;
        }
        Ok(())
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
    fn shift_in_range<R: ShiftRange>(&mut self, range: &R, by: (isize, isize)) {
        self.ctx = None;
        self.cases.shift_in_range(range, by);
        self.inferred_type = None;
    }
}

impl Occurs for LocalComatch {
    fn occurs(&self, ctx: &mut LevelCtx, lvl: Lvl) -> bool {
        let LocalComatch { cases, .. } = self;
        cases.occurs(ctx, lvl)
    }
}

impl HasType for LocalComatch {
    fn typ(&self) -> Option<Box<Exp>> {
        self.inferred_type.clone().map(|x| Box::new(x.into()))
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

impl Print for LocalComatch {
    fn print_prec<'a>(
        &'a self,
        cfg: &PrintCfg,
        alloc: &'a Alloc<'a>,
        _prec: Precedence,
    ) -> Builder<'a> {
        let LocalComatch { name, is_lambda_sugar, cases, .. } = self;
        if *is_lambda_sugar && cfg.print_lambda_sugar {
            print_lambda_sugar(cases, cfg, alloc)
        } else {
            alloc
                .keyword(COMATCH)
                .append(match &name.user_name {
                    Some(name) => alloc.space().append(alloc.ctor(&name.id)),
                    None => alloc.nil(),
                })
                .append(alloc.space())
                .append(print_cases(cases, cfg, alloc))
        }
    }
}

impl Zonk for LocalComatch {
    fn zonk(
        &mut self,
        meta_vars: &crate::HashMap<MetaVar, crate::MetaVarState>,
    ) -> Result<(), ZonkError> {
        let LocalComatch { span: _, ctx: _, name: _, is_lambda_sugar: _, cases, inferred_type } =
            self;
        inferred_type.zonk(meta_vars)?;
        for case in cases {
            case.zonk(meta_vars)?;
        }
        Ok(())
    }
}

/// Print the Comatch as a lambda abstraction.
/// Only invoke this function if the comatch contains exactly
/// one cocase "ap" with three arguments; the function will
/// panic otherwise.
fn print_lambda_sugar<'a>(cases: &'a [Case], cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
    let Case { pattern, body, .. } = cases.first().expect("Empty comatch marked as lambda sugar");
    let var_name = pattern
        .params
        .params
        .get(2) // The variable we want to print is at the third position: comatch { ap(_,_,x) => ...}
        .expect("No parameter bound in comatch marked as lambda sugar")
        .name();
    alloc
        .backslash_anno(cfg)
        .append(&var_name.id)
        .append(DOT)
        .append(alloc.space())
        .append(body.print(cfg, alloc))
}

// Hole
//
//

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Hole {
    /// Source code location
    pub span: Option<Span>,
    /// Whether the hole must be solved during typechecking or not.
    pub kind: MetaVarKind,
    /// The metavariable that we want to solve for that hole
    pub metavar: MetaVar,
    /// The inferred type of the hole annotated during elaboration.
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub inferred_type: Option<Box<Exp>>,
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
    pub args: Vec<Vec<Box<Exp>>>,
    /// The solution found by unification. It is propagated during zonking.
    pub solution: Option<Box<Exp>>,
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
    fn shift_in_range<R: ShiftRange>(&mut self, range: &R, by: (isize, isize)) {
        let Hole { span: _, kind: _, metavar: _, inferred_type, inferred_ctx, args, solution } =
            self;

        *inferred_type = None;
        *inferred_ctx = None;
        args.shift_in_range(range, by);
        solution.shift_in_range(range, by);
    }
}

impl Occurs for Hole {
    fn occurs(&self, _ctx: &mut LevelCtx, _lvl: Lvl) -> bool {
        false
    }
}

impl HasType for Hole {
    fn typ(&self) -> Option<Box<Exp>> {
        self.inferred_type.clone()
    }
}

impl Substitutable for Hole {
    type Result = Hole;

    fn subst<S: Substitution>(&self, ctx: &mut LevelCtx, by: &S) -> Self::Result {
        let Hole { span, kind, metavar, args, .. } = self;
        Hole {
            span: *span,
            kind: *kind,
            metavar: *metavar,
            inferred_type: None,
            inferred_ctx: None,
            args: args.subst(ctx, by),
            solution: self.solution.subst(ctx, by),
        }
    }
}

impl Print for Hole {
    fn print_prec<'a>(
        &'a self,
        cfg: &PrintCfg,
        alloc: &'a Alloc<'a>,
        prec: Precedence,
    ) -> Builder<'a> {
        match self.kind {
            MetaVarKind::MustSolve => {
                let mut doc = alloc.keyword(UNDERSCORE);

                if cfg.print_metavar_ids {
                    doc = doc.append(self.metavar.id.to_string());
                }

                if let Some(solution) = &self.solution {
                    doc = doc.append("<").append(solution.print_prec(cfg, alloc, prec)).append(">")
                }

                doc
            }
            MetaVarKind::CanSolve => {
                let mut doc = alloc.keyword(QUESTION_MARK);

                if cfg.print_metavar_ids {
                    doc = doc.append(self.metavar.id.to_string());
                }

                if let Some(solution) = &self.solution {
                    doc = doc.append("<").append(solution.print_prec(cfg, alloc, prec)).append(">")
                }

                doc
            }
            MetaVarKind::Inserted => {
                let mut doc = alloc.nil();

                if cfg.print_metavar_ids {
                    doc = doc.append(self.metavar.id.to_string());
                }

                match &self.solution {
                    Some(solution) => {
                        doc = doc
                            .append("<")
                            .append(solution.print_prec(cfg, alloc, prec))
                            .append(">")
                    }
                    None => doc = doc.append("<Inserted>"),
                }

                doc
            }
        }
    }
}

impl Zonk for Hole {
    fn zonk(
        &mut self,
        meta_vars: &crate::HashMap<MetaVar, crate::MetaVarState>,
    ) -> Result<(), ZonkError> {
        match meta_vars.get(&self.metavar) {
            Some(crate::MetaVarState::Solved { ctx, solution }) => {
                self.solution = Some(solution.subst_under_ctx(ctx.levels(), &self.args));
            }
            Some(crate::MetaVarState::Unsolved { .. }) => {
                // Nothing to do, the hole remains unsolved
            }
            None => {
                return Err(ZonkError::UnboundMetaVar(self.metavar));
            }
        }

        Ok(())
    }
}

// Pattern
//
//

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Pattern {
    pub is_copattern: bool,
    pub name: Ident,
    pub params: TelescopeInst,
}

impl Print for Pattern {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Pattern { is_copattern, name, params } = self;
        if *is_copattern {
            alloc.text(DOT).append(alloc.ctor(&name.id)).append(params.print(cfg, alloc))
        } else {
            alloc.ctor(&name.id).append(params.print(cfg, alloc))
        }
    }
}

impl Zonk for Pattern {
    fn zonk(
        &mut self,
        meta_vars: &crate::HashMap<MetaVar, crate::MetaVarState>,
    ) -> Result<(), ZonkError> {
        let Pattern { is_copattern: _, name: _, params } = self;
        params.zonk(meta_vars)
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
    pub pattern: Pattern,
    /// Body being `None` represents an absurd pattern
    pub body: Option<Box<Exp>>,
}

impl Shift for Case {
    fn shift_in_range<R: ShiftRange>(&mut self, range: &R, by: (isize, isize)) {
        self.body.shift_in_range(&range.clone().shift(1), by);
    }
}

impl Occurs for Case {
    fn occurs(&self, ctx: &mut LevelCtx, lvl: Lvl) -> bool {
        let Case { pattern, body, .. } = self;
        ctx.bind_iter(pattern.params.params.iter().map(|_| ()), |ctx| body.occurs(ctx, lvl))
    }
}

impl Substitutable for Case {
    type Result = Case;
    fn subst<S: Substitution>(&self, ctx: &mut LevelCtx, by: &S) -> Self {
        let Case { span, pattern, body } = self;
        ctx.bind_iter(pattern.params.params.iter(), |ctx| Case {
            span: *span,
            pattern: pattern.clone(),
            body: body.as_ref().map(|body| {
                let mut by = (*by).clone();
                by.shift((1, 0));
                body.subst(ctx, &by)
            }),
        })
    }
}

impl Print for Case {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Case { span: _, pattern, body } = self;

        let body = match body {
            None => alloc.keyword(ABSURD),
            Some(body) => alloc
                .text(FAT_ARROW)
                .append(alloc.line())
                .append(body.print(cfg, alloc))
                .nest(cfg.indent),
        };

        pattern.print(cfg, alloc).append(alloc.space()).append(body).group()
    }
}

impl Zonk for Case {
    fn zonk(
        &mut self,
        meta_vars: &crate::HashMap<MetaVar, crate::MetaVarState>,
    ) -> Result<(), ZonkError> {
        let Case { span: _, pattern, body } = self;
        pattern.zonk(meta_vars)?;
        body.zonk(meta_vars)?;
        Ok(())
    }
}

pub fn print_cases<'a>(cases: &'a [Case], cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
    match cases.len() {
        0 => empty_braces(alloc),

        1 => alloc
            .line()
            .append(cases[0].print(cfg, alloc))
            .nest(cfg.indent)
            .append(alloc.line())
            .braces_anno()
            .group(),
        _ => {
            let sep = alloc.text(COMMA).append(alloc.hardline());
            alloc
                .hardline()
                .append(alloc.intersperse(cases.iter().map(|x| x.print(cfg, alloc)), sep.clone()))
                .nest(cfg.indent)
                .append(alloc.hardline())
                .braces_anno()
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

impl Print for TelescopeInst {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        if self.params.is_empty() {
            alloc.nil()
        } else {
            self.params.print(cfg, alloc).parens()
        }
    }
}

impl Zonk for TelescopeInst {
    fn zonk(
        &mut self,
        meta_vars: &crate::HashMap<MetaVar, crate::MetaVarState>,
    ) -> Result<(), ZonkError> {
        let TelescopeInst { params } = self;

        for param in params {
            param.zonk(meta_vars)?;
        }
        Ok(())
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
    pub info: Option<Box<Exp>>,
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub name: Ident,
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub typ: Option<Box<Exp>>,
}

impl Named for ParamInst {
    fn name(&self) -> &Ident {
        &self.name
    }
}

impl Print for ParamInst {
    fn print<'a>(&'a self, _cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let ParamInst { span: _, info: _, name, typ: _ } = self;
        alloc.text(&name.id)
    }
}

impl Zonk for ParamInst {
    fn zonk(
        &mut self,
        meta_vars: &crate::HashMap<MetaVar, crate::MetaVarState>,
    ) -> Result<(), ZonkError> {
        let ParamInst { span: _, info, name: _, typ } = self;

        info.zonk(meta_vars)?;
        typ.zonk(meta_vars)?;
        Ok(())
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
    pub args: Vec<Arg>,
}

impl Args {
    pub fn to_exps(&self) -> Vec<Box<Exp>> {
        self.args.iter().map(|arg| arg.exp().clone()).collect()
    }

    pub fn len(&self) -> usize {
        self.args.len()
    }

    pub fn is_empty(&self) -> bool {
        self.args.is_empty()
    }
}

impl Shift for Args {
    fn shift_in_range<R: ShiftRange>(&mut self, range: &R, by: (isize, isize)) {
        self.args.shift_in_range(range, by);
    }
}

impl Substitutable for Args {
    type Result = Args;
    fn subst<S: Substitution>(&self, ctx: &mut LevelCtx, by: &S) -> Self {
        Args { args: self.args.subst(ctx, by) }
    }
}

impl Print for Args {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let mut doc = alloc.nil();
        let mut first = true;

        for arg in &self.args {
            if !arg.is_inserted_implicit() {
                if !first {
                    doc = doc.append(COMMA).append(alloc.line());
                }
                doc = doc.append(arg.print(cfg, alloc));
                first = false;
            }
        }

        doc.align().parens().group()
    }
}

impl Zonk for Args {
    fn zonk(
        &mut self,
        meta_vars: &crate::HashMap<MetaVar, crate::MetaVarState>,
    ) -> Result<(), ZonkError> {
        let Args { args } = self;

        for arg in args {
            arg.zonk(meta_vars)?;
        }
        Ok(())
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
    pub ret_typ: Box<Exp>,
}

impl Shift for Motive {
    fn shift_in_range<R: ShiftRange>(&mut self, range: &R, by: (isize, isize)) {
        self.ret_typ.shift_in_range(&range.clone().shift(1), by);
    }
}

impl Substitutable for Motive {
    type Result = Motive;
    fn subst<S: Substitution>(&self, ctx: &mut LevelCtx, by: &S) -> Self::Result {
        let Motive { span, param, ret_typ } = self;

        Motive {
            span: *span,
            param: param.clone(),
            ret_typ: ctx.bind_single((), |ctx| {
                let mut by = (*by).clone();
                by.shift((1, 0));
                ret_typ.subst(ctx, &by)
            }),
        }
    }
}

impl Print for Motive {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Motive { span: _, param, ret_typ } = self;

        alloc
            .space()
            .append(alloc.keyword(AS))
            .append(alloc.space())
            .append(param.print(cfg, alloc))
            .append(alloc.space())
            .append(alloc.text(FAT_ARROW))
            .append(alloc.space())
            .append(ret_typ.print(cfg, alloc))
    }
}

impl Zonk for Motive {
    fn zonk(
        &mut self,
        meta_vars: &crate::HashMap<MetaVar, crate::MetaVarState>,
    ) -> Result<(), ZonkError> {
        let Motive { span: _, param, ret_typ } = self;
        param.zonk(meta_vars)?;
        ret_typ.zonk(meta_vars)?;
        Ok(())
    }
}
