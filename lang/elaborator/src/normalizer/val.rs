use std::rc::Rc;

use ast;
use ast::ctx::BindContext;
use ast::Idx;
use ast::MetaVar;
use ast::Shift;
use ast::ShiftRange;
use ast::ShiftRangeExt;
use codespan::Span;
use derivative::Derivative;
use log::trace;
use pretty::DocAllocator;
use printer::theme::ThemeExt;
use printer::tokens::*;
use printer::types::Print;
use printer::types::*;
use printer::util::*;

use crate::normalizer::env::*;

use super::eval::Eval;
use crate::result::*;

fn print_cases<'a>(cases: &'a [Case], cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
    let sep = alloc.text(COMMA).append(alloc.hardline());
    alloc
        .hardline()
        .append(alloc.intersperse(cases.iter().map(|x| x.print(cfg, alloc)), sep))
        .nest(cfg.indent)
        .append(alloc.hardline())
        .braces_anno()
}

// Read back
//
//

/// Every value and neutral term defined in this module
/// corresponds to an expression in normal form.
/// This trait allows to convert values and neutral terms back to expressions.
pub trait ReadBack {
    type Nf;

    fn read_back(&self, prg: &ast::Module) -> Result<Self::Nf, TypeError>;
}

impl<T: ReadBack> ReadBack for Vec<T> {
    type Nf = Vec<T::Nf>;

    fn read_back(&self, prg: &ast::Module) -> Result<Self::Nf, TypeError> {
        self.iter().map(|x| x.read_back(prg)).collect()
    }
}

impl<T: ReadBack> ReadBack for Rc<T> {
    type Nf = Box<T::Nf>;

    fn read_back(&self, prg: &ast::Module) -> Result<Self::Nf, TypeError> {
        (**self).read_back(prg).map(Box::new)
    }
}

impl<T: ReadBack> ReadBack for Option<T> {
    type Nf = Option<T::Nf>;

    fn read_back(&self, prg: &ast::Module) -> Result<Self::Nf, TypeError> {
        self.as_ref().map(|x| x.read_back(prg)).transpose()
    }
}

// Val
//
//

/// The result of evaluation
#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub enum Val {
    TypCtor(TypCtor),
    // A call is only a value if it is a constructor or a codefinition.
    Call(Call),
    TypeUniv(TypeUniv),
    LocalComatch(LocalComatch),
    Neu(Neu),
}

impl Shift for Val {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        match self {
            Val::TypCtor(e) => e.shift_in_range(range, by).into(),
            Val::Call(e) => e.shift_in_range(range, by).into(),
            Val::TypeUniv(e) => e.shift_in_range(range, by).into(),
            Val::LocalComatch(e) => e.shift_in_range(range, by).into(),
            Val::Neu(exp) => exp.shift_in_range(range, by).into(),
        }
    }
}

impl Print for Val {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        match self {
            Val::TypCtor(e) => e.print(cfg, alloc),
            Val::Call(e) => e.print(cfg, alloc),
            Val::TypeUniv(e) => e.print(cfg, alloc),
            Val::LocalComatch(e) => e.print(cfg, alloc),
            Val::Neu(exp) => exp.print(cfg, alloc),
        }
    }
}

impl ReadBack for Val {
    type Nf = ast::Exp;

    fn read_back(&self, prg: &ast::Module) -> Result<Self::Nf, TypeError> {
        let res = match self {
            Val::TypCtor(e) => e.read_back(prg)?.into(),
            Val::Call(e) => e.read_back(prg)?.into(),
            Val::TypeUniv(e) => e.read_back(prg)?.into(),
            Val::LocalComatch(e) => e.read_back(prg)?.into(),
            Val::Neu(exp) => exp.read_back(prg)?,
        };
        trace!("↓{} ~> {}", self.print_trace(), res.print_trace());
        Ok(res)
    }
}

// TypCtor
//
//

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct TypCtor {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub span: Option<Span>,
    pub name: ast::Ident,
    pub args: Args,
}

impl Shift for TypCtor {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let TypCtor { span, name, args } = self;
        TypCtor { span: *span, name: name.clone(), args: args.shift_in_range(range, by) }
    }
}

impl Print for TypCtor {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let TypCtor { span: _, name, args } = self;
        let psubst = if args.is_empty() { alloc.nil() } else { args.print(cfg, alloc).parens() };
        alloc.typ(name).append(psubst)
    }
}

impl From<TypCtor> for Val {
    fn from(value: TypCtor) -> Self {
        Val::TypCtor(value)
    }
}

impl ReadBack for TypCtor {
    type Nf = ast::TypCtor;

    fn read_back(&self, prg: &ast::Module) -> Result<Self::Nf, TypeError> {
        let TypCtor { span, name, args } = self;
        Ok(ast::TypCtor {
            span: *span,
            name: name.clone(),
            args: ast::Args { args: args.read_back(prg)? },
        })
    }
}

// Call
//
//

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Call {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub span: Option<Span>,
    pub kind: ast::CallKind,
    pub name: ast::Ident,
    pub args: Args,
}

impl Shift for Call {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let Call { span, kind, name, args } = self;
        Call { span: *span, kind: *kind, name: name.clone(), args: args.shift_in_range(range, by) }
    }
}

impl Print for Call {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Call { span: _, kind: _, name, args } = self;
        let psubst = if args.is_empty() { alloc.nil() } else { args.print(cfg, alloc).parens() };
        alloc.ctor(name).append(psubst)
    }
}

impl From<Call> for Val {
    fn from(value: Call) -> Self {
        Val::Call(value)
    }
}

impl ReadBack for Call {
    type Nf = ast::Call;

    fn read_back(&self, prg: &ast::Module) -> Result<Self::Nf, TypeError> {
        let Call { span, kind, name, args } = self;
        Ok(ast::Call {
            span: *span,
            kind: *kind,
            name: name.clone(),
            args: ast::Args { args: args.read_back(prg)? },
            inferred_type: None,
        })
    }
}

// TypeUniv
//
//

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct TypeUniv {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub span: Option<Span>,
}

impl Shift for TypeUniv {
    fn shift_in_range<R: ShiftRange>(&self, _range: R, _by: (isize, isize)) -> Self {
        self.clone()
    }
}

impl Print for TypeUniv {
    fn print<'a>(&'a self, _cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        alloc.typ(TYPE)
    }
}

impl From<TypeUniv> for Val {
    fn from(value: TypeUniv) -> Self {
        Val::TypeUniv(value)
    }
}

impl ReadBack for TypeUniv {
    type Nf = ast::TypeUniv;

    fn read_back(&self, _prg: &ast::Module) -> Result<Self::Nf, TypeError> {
        let TypeUniv { span } = self;
        Ok(ast::TypeUniv { span: *span })
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
    pub name: ast::Label,
    pub is_lambda_sugar: bool,
    pub cases: Vec<Case>,
}

impl Shift for LocalComatch {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let LocalComatch { span, name, is_lambda_sugar, cases } = self;
        LocalComatch {
            span: *span,
            name: name.clone(),
            is_lambda_sugar: *is_lambda_sugar,
            cases: cases.shift_in_range(range, by),
        }
    }
}

impl Print for LocalComatch {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let LocalComatch { span: _, name, is_lambda_sugar: _, cases } = self;
        alloc
            .keyword(COMATCH)
            .append(alloc.space())
            .append(alloc.text(name.to_string()))
            .append(alloc.space())
            .append(print_cases(cases, cfg, alloc))
    }
}

impl From<LocalComatch> for Val {
    fn from(value: LocalComatch) -> Self {
        Val::LocalComatch(value)
    }
}

impl ReadBack for LocalComatch {
    type Nf = ast::LocalComatch;
    fn read_back(&self, prg: &ast::Module) -> Result<Self::Nf, TypeError> {
        let LocalComatch { span, name, is_lambda_sugar, cases } = self;
        Ok(ast::LocalComatch {
            span: *span,
            ctx: None,
            name: name.clone(),
            is_lambda_sugar: *is_lambda_sugar,
            cases: cases.read_back(prg)?,
            inferred_type: None,
        })
    }
}

// Neu
//
//

/// A term whose evaluation is blocked
#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub enum Neu {
    Variable(Variable),
    DotCall(DotCall),
    LocalMatch(LocalMatch),
    Hole(Hole),
    /// A call which corresponds to an opaque let-bound definition on the toplevel
    /// cannot be inlined and must therefore block computation.
    OpaqueCall(OpaqueCall),
}

impl Shift for Neu {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        match self {
            Neu::Variable(e) => e.shift_in_range(range, by).into(),
            Neu::DotCall(e) => e.shift_in_range(range, by).into(),
            Neu::LocalMatch(e) => e.shift_in_range(range, by).into(),
            Neu::Hole(e) => e.shift_in_range(range, by).into(),
            Neu::OpaqueCall(e) => e.shift_in_range(range, by).into(),
        }
    }
}

impl Print for Neu {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        match self {
            Neu::Variable(e) => e.print(cfg, alloc),
            Neu::DotCall(e) => e.print(cfg, alloc),
            Neu::LocalMatch(e) => e.print(cfg, alloc),
            Neu::Hole(e) => e.print(cfg, alloc),
            Neu::OpaqueCall(e) => e.print(cfg, alloc),
        }
    }
}

impl From<Neu> for Val {
    fn from(value: Neu) -> Self {
        Val::Neu(value)
    }
}

impl ReadBack for Neu {
    type Nf = ast::Exp;

    fn read_back(&self, prg: &ast::Module) -> Result<Self::Nf, TypeError> {
        let res = match self {
            Neu::Variable(e) => e.read_back(prg)?.into(),
            Neu::DotCall(e) => e.read_back(prg)?.into(),
            Neu::LocalMatch(e) => e.read_back(prg)?.into(),
            Neu::Hole(e) => e.read_back(prg)?.into(),
            Neu::OpaqueCall(e) => e.read_back(prg)?.into(),
        };
        Ok(res)
    }
}

// Variable
//
//

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Variable {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub span: Option<Span>,
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub name: ast::Ident,
    pub idx: Idx,
}

impl Shift for Variable {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let Variable { span, name, idx } = self;
        Variable { span: *span, name: name.clone(), idx: idx.shift_in_range(range, by) }
    }
}

impl Print for Variable {
    fn print<'a>(&'a self, _cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Variable { span: _, name, idx } = self;
        alloc.text(format!("{name}@{idx}"))
    }
}

impl From<Variable> for Neu {
    fn from(value: Variable) -> Self {
        Neu::Variable(value)
    }
}

impl ReadBack for Variable {
    type Nf = ast::Variable;

    fn read_back(&self, _prg: &ast::Module) -> Result<Self::Nf, TypeError> {
        let Variable { span, name, idx } = self;
        Ok(ast::Variable { span: *span, idx: *idx, name: name.clone(), inferred_type: None })
    }
}

// DotCall
//
//

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct DotCall {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub span: Option<Span>,
    pub kind: ast::DotCallKind,
    pub exp: Rc<Neu>,
    pub name: ast::Ident,
    pub args: Args,
}

impl Shift for DotCall {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let DotCall { span, kind, exp, name, args } = self;
        DotCall {
            span: *span,
            kind: *kind,
            exp: exp.shift_in_range(range.clone(), by),
            name: name.clone(),
            args: args.shift_in_range(range, by),
        }
    }
}

impl Print for DotCall {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let DotCall { span: _, kind: _, exp, name, args } = self;
        let psubst = if args.is_empty() { alloc.nil() } else { args.print(cfg, alloc).parens() };
        exp.print(cfg, alloc).append(DOT).append(alloc.dtor(name)).append(psubst)
    }
}

impl From<DotCall> for Neu {
    fn from(value: DotCall) -> Self {
        Neu::DotCall(value)
    }
}

impl ReadBack for DotCall {
    type Nf = ast::DotCall;

    fn read_back(&self, prg: &ast::Module) -> Result<Self::Nf, TypeError> {
        let DotCall { span, kind, exp, name, args } = self;
        Ok(ast::DotCall {
            span: *span,
            kind: *kind,
            exp: exp.read_back(prg)?,
            name: name.clone(),
            args: ast::Args { args: args.read_back(prg)? },
            inferred_type: None,
        })
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
    pub name: ast::Label,
    pub on_exp: Rc<Neu>,
    pub cases: Vec<Case>,
}

impl Shift for LocalMatch {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let LocalMatch { span, name, on_exp, cases } = self;
        LocalMatch {
            span: *span,
            name: name.clone(),
            on_exp: on_exp.shift_in_range(range.clone(), by),
            cases: cases.shift_in_range(range, by),
        }
    }
}

impl Print for LocalMatch {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let LocalMatch { span: _, name, on_exp, cases } = self;
        on_exp
            .print(cfg, alloc)
            .append(DOT)
            .append(alloc.keyword(MATCH))
            .append(alloc.space())
            .append(alloc.text(name.to_string()))
            .append(alloc.space())
            .append(print_cases(cases, cfg, alloc))
    }
}

impl From<LocalMatch> for Neu {
    fn from(value: LocalMatch) -> Self {
        Neu::LocalMatch(value)
    }
}

impl ReadBack for LocalMatch {
    type Nf = ast::LocalMatch;

    fn read_back(&self, prg: &ast::Module) -> Result<Self::Nf, TypeError> {
        let LocalMatch { span, name, on_exp, cases } = self;
        Ok(ast::LocalMatch {
            span: *span,
            ctx: None,
            motive: None,
            ret_typ: None,
            name: name.clone(),
            on_exp: on_exp.read_back(prg)?,
            cases: cases.read_back(prg)?,
            inferred_type: None,
        })
    }
}

// Hole
//
//

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Hole {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub span: Option<Span>,
    pub kind: ast::HoleKind,
    pub metavar: MetaVar,
    /// Explicit substitution of the context, compare documentation of ast::Hole
    pub args: Vec<Vec<Rc<Val>>>,
}

impl Shift for Hole {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let Hole { span, kind, metavar, args } = self;
        Hole {
            span: *span,
            kind: kind.clone(),
            metavar: *metavar,
            args: args.shift_in_range(range, by),
        }
    }
}

impl Print for Hole {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        if cfg.print_metavar_ids {
            alloc.text(format!("?{}", self.metavar.id))
        } else {
            alloc.keyword(QUESTIONMARK)
        }
    }
}

impl From<Hole> for Neu {
    fn from(value: Hole) -> Self {
        Neu::Hole(value)
    }
}

impl ReadBack for Hole {
    type Nf = ast::Hole;

    fn read_back(&self, prg: &ast::Module) -> Result<Self::Nf, TypeError> {
        let Hole { span, kind, metavar, args } = self;
        let args = args.read_back(prg)?;
        Ok(ast::Hole {
            span: *span,
            kind: kind.clone(),
            metavar: *metavar,
            inferred_type: None,
            inferred_ctx: None,
            args,
        })
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
    pub is_copattern: bool,
    pub name: ast::Ident,
    pub params: ast::TelescopeInst,
    /// Body being `None` represents an absurd pattern
    pub body: Option<Closure>,
}

impl Shift for Case {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let Case { span, is_copattern, name, params, body } = self;

        Case {
            span: *span,
            is_copattern: *is_copattern,
            name: name.clone(),
            params: params.clone(),
            body: body.shift_in_range(range.shift(1), by),
        }
    }
}

impl Print for Case {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Case { span: _, is_copattern: _, name, params, body } = self;

        let body = match body {
            None => alloc.keyword(ABSURD),
            Some(body) => alloc
                .text(FAT_ARROW)
                .append(alloc.line())
                .append(body.print(cfg, alloc))
                .nest(cfg.indent),
        };

        alloc.ctor(name).append(params.print(cfg, alloc)).append(alloc.space()).append(body).group()
    }
}

impl ReadBack for Case {
    type Nf = ast::Case;

    fn read_back(&self, prg: &ast::Module) -> Result<Self::Nf, TypeError> {
        let Case { span, is_copattern, name, params, body } = self;

        Ok(ast::Case {
            span: *span,
            pattern: ast::Pattern {
                is_copattern: *is_copattern,
                name: name.clone(),
                params: params.clone(),
            },
            body: body.read_back(prg)?,
        })
    }
}

// OpaqueCall
//
//

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct OpaqueCall {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub span: Option<Span>,
    pub name: ast::Ident,
    pub args: Args,
}

impl Shift for OpaqueCall {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let OpaqueCall { span, name, args } = self;
        OpaqueCall { span: *span, name: name.clone(), args: args.shift_in_range(range, by) }
    }
}

impl Print for OpaqueCall {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let OpaqueCall { span: _, name, args } = self;
        let psubst = if args.is_empty() { alloc.nil() } else { args.print(cfg, alloc).parens() };
        alloc.ctor(name).append(psubst)
    }
}

impl From<OpaqueCall> for Neu {
    fn from(value: OpaqueCall) -> Self {
        Neu::OpaqueCall(value)
    }
}

impl ReadBack for OpaqueCall {
    type Nf = ast::Call;

    fn read_back(&self, prg: &ast::Module) -> Result<Self::Nf, TypeError> {
        let OpaqueCall { span, name, args } = self;
        Ok(ast::Call {
            span: *span,
            kind: ast::CallKind::LetBound,
            name: name.clone(),
            args: ast::Args { args: args.read_back(prg)? },
            inferred_type: None,
        })
    }
}

// Args
//
//

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Args(pub Vec<Arg>);

impl Args {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn to_vals(&self) -> Vec<Rc<Val>> {
        self.0.iter().map(Arg::to_val).collect()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Arg> {
        self.0.iter()
    }
}

impl Shift for Args {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        Args(self.0.shift_in_range(range, by))
    }
}

impl ReadBack for Args {
    type Nf = Vec<ast::Arg>;

    fn read_back(&self, prg: &ast::Module) -> Result<Self::Nf, TypeError> {
        self.0.read_back(prg)
    }
}

impl Print for Args {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let mut doc = alloc.nil();
        let mut first = true;

        for arg in &self.0 {
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

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub enum Arg {
    UnnamedArg(Rc<Val>),
    NamedArg(ast::Ident, Rc<Val>),
    InsertedImplicitArg(Rc<Val>),
}

impl Arg {
    pub fn is_inserted_implicit(&self) -> bool {
        matches!(self, Arg::InsertedImplicitArg(_))
    }
}

impl Shift for Arg {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        match self {
            Arg::UnnamedArg(val) => Arg::UnnamedArg(val.shift_in_range(range, by)),
            Arg::NamedArg(name, val) => Arg::NamedArg(name.clone(), val.shift_in_range(range, by)),
            Arg::InsertedImplicitArg(val) => {
                Arg::InsertedImplicitArg(val.shift_in_range(range, by))
            }
        }
    }
}

impl ReadBack for Arg {
    type Nf = ast::Arg;

    fn read_back(&self, prg: &ast::Module) -> Result<Self::Nf, TypeError> {
        match self {
            Arg::UnnamedArg(val) => Ok(ast::Arg::UnnamedArg(val.read_back(prg)?)),
            Arg::NamedArg(name, val) => Ok(ast::Arg::NamedArg(name.clone(), val.read_back(prg)?)),
            Arg::InsertedImplicitArg(val) => Ok(ast::Arg::UnnamedArg(val.read_back(prg)?)),
        }
    }
}

impl Print for Arg {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        match self {
            Arg::UnnamedArg(val) => val.print(cfg, alloc),
            Arg::NamedArg(name, val) => alloc
                .text(name.to_string())
                .append(alloc.space())
                .append(COLONEQ)
                .append(alloc.space())
                .append(val.print(cfg, alloc)),
            Arg::InsertedImplicitArg(_) => {
                panic!("Inserted implicit arguments should not be printed")
            }
        }
    }
}

impl Arg {
    pub fn to_val(&self) -> Rc<Val> {
        match self {
            Arg::UnnamedArg(val) => val.clone(),
            Arg::NamedArg(_, val) => val.clone(),
            Arg::InsertedImplicitArg(val) => val.clone(),
        }
    }
}

// Closure
//
//

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Closure {
    pub env: Env,
    pub n_args: usize,
    pub body: Box<ast::Exp>,
}

impl Shift for Closure {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let Closure { env, n_args, body } = self;

        Closure { env: env.shift_in_range(range, by), n_args: *n_args, body: body.clone() }
    }
}

impl Print for Closure {
    fn print<'a>(&'a self, _cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        alloc.text("...")
    }
}

impl ReadBack for Closure {
    type Nf = Box<ast::Exp>;

    fn read_back(&self, prg: &ast::Module) -> Result<Self::Nf, TypeError> {
        let args: Vec<Rc<Val>> = (0..self.n_args)
            .rev()
            .map(|snd| {
                Val::Neu(Neu::Variable(Variable {
                    span: None,
                    name: "".to_owned(),
                    idx: Idx { fst: 0, snd },
                }))
            })
            .map(Rc::new)
            .collect();
        self.env
            .shift((1, 0))
            .bind_iter(args.iter(), |env| self.body.eval(prg, env))?
            .read_back(prg)
    }
}
