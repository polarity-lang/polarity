use miette_util::codespan::Span;
use num_bigint::BigUint;

use super::ident::*;

#[derive(Debug, Clone)]
pub enum BindingSite {
    Var { span: Span, name: Ident },
    Wildcard { span: Span },
}

impl BindingSite {
    pub fn span(&self) -> Span {
        match self {
            BindingSite::Var { span, .. } => *span,
            BindingSite::Wildcard { span } => *span,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Pattern {
    pub span: Span,
    pub name: Ident,
    pub params: Vec<BindingSite>,
}

#[derive(Debug, Clone)]
pub struct Copattern {
    pub span: Span,
    pub name: Ident,
    pub params: Vec<BindingSite>,
}

#[derive(Debug, Clone)]
pub struct Case<P> {
    pub span: Span,
    pub pattern: P,
    /// Body being `None` represents an absurd pattern
    pub body: Option<Box<Exp>>,
}

/// Arguments in an argument list can either be unnamed or named.
/// Example for named arguments: `f(x := 1, y := 2)`
/// Example for unnamed arguments: `f(1, 2)``
#[derive(Debug, Clone)]
pub enum Arg {
    UnnamedArg(Box<Exp>),
    NamedArg(Ident, Box<Exp>),
}

impl Arg {
    pub fn span(&self) -> Span {
        match self {
            Arg::UnnamedArg(exp) => exp.span(),
            Arg::NamedArg(_, exp) => exp.span(),
        }
    }

    pub fn is_underscore(&self) -> bool {
        match self {
            Arg::UnnamedArg(arg) => arg.is_underscore(),
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Exp {
    Call(Call),
    DotCall(DotCall),
    Anno(Anno),
    LocalMatch(LocalMatch),
    LocalComatch(LocalComatch),
    Hole(Hole),
    NatLit(NatLit),
    BinOp(BinOp),
    Lam(Lam),
}

impl Exp {
    pub fn span(&self) -> Span {
        match self {
            Exp::Call(call) => call.span,
            Exp::DotCall(dot_call) => dot_call.span,
            Exp::Anno(anno) => anno.span,
            Exp::LocalMatch(local_match) => local_match.span,
            Exp::LocalComatch(local_comatch) => local_comatch.span,
            Exp::Hole(hole) => hole.span,
            Exp::NatLit(nat_lit) => nat_lit.span,
            Exp::BinOp(binop) => binop.span,
            Exp::Lam(lam) => lam.span,
        }
    }

    /// Checks whether the expression is a hole `_` written with an underscore.
    pub fn is_underscore(&self) -> bool {
        match self {
            Exp::Hole(h) => match h.kind {
                HoleKind::MustSolve => true,
                _ => false,
            },
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
/// Either a constructor or call of a toplevel let or a variable
pub struct Call {
    pub span: Span,
    pub name: Ident,
    pub args: Vec<Arg>,
}

#[derive(Debug, Clone)]
/// Something of the form a.b or a.b(c, ..)
pub struct DotCall {
    pub span: Span,
    pub exp: Box<Exp>,
    pub name: Ident,
    pub args: Vec<Arg>,
}

#[derive(Debug, Clone)]
/// Type annotations like e : e
pub struct Anno {
    pub span: Span,
    pub exp: Box<Exp>,
    pub typ: Box<Exp>,
}

#[derive(Debug, Clone)]
/// Pattern match, e.g. expr.match { A => .., ..}
pub struct LocalMatch {
    pub span: Span,
    pub name: Option<Ident>,
    pub on_exp: Box<Exp>,
    pub motive: Option<Motive>,
    pub cases: Vec<Case<Pattern>>,
}

#[derive(Debug, Clone)]
/// Copattern match, e.g. comatch { .x(a, ..) => .. }
pub struct LocalComatch {
    pub span: Span,
    pub name: Option<Ident>,
    pub is_lambda_sugar: bool,
    pub cases: Vec<Case<Copattern>>,
}

#[derive(Debug, Clone)]
pub enum HoleKind {
    /// A hole `_` that must be solved by the constraint solver.
    MustSolve,
    /// A hole `?` that the programmer needs help with.
    CanSolve,
}

#[derive(Debug, Clone)]
pub struct Hole {
    pub span: Span,
    pub kind: HoleKind,
}

#[derive(Debug, Clone)]
/// Literal for a natural number
pub struct NatLit {
    pub span: Span,
    pub val: BigUint,
}

#[derive(Debug, Clone)]
/// Binary Operator, e.g. `e -> e` or `e + e`.
pub struct BinOp {
    pub span: Span,
    pub operator: Operator,
    pub lhs: Box<Exp>,
    pub rhs: Box<Exp>,
}

#[derive(Debug, Clone)]
/// Syntactic sugar for codata types with only one observation.
/// Java, for example, calls these "SAM-types", i.e. types with a "single abstract method".
/// The standard example of this is the function type, which has the destructor "ap".
pub struct Lam {
    pub span: Span,
    pub case: Case<Copattern>,
}

#[derive(Debug, Clone)]
pub struct Motive {
    pub span: Span,
    pub param: BindingSite,
    pub ret_typ: Box<Exp>,
}
