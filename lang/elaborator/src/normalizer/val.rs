use std::rc::Rc;

use derivative::Derivative;
use syntax::ast::MetaVar;

use crate::normalizer::env::*;
use codespan::Span;
use pretty::DocAllocator;
use printer::theme::ThemeExt;
use printer::tokens::*;
use printer::types::*;
use printer::util::*;
use syntax::ast;
use syntax::common::*;
use syntax::ctx::BindContext;
use tracer::trace;

use super::eval::Eval;
use crate::result::*;

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
    type Nf = Rc<T::Nf>;

    fn read_back(&self, prg: &ast::Module) -> Result<Self::Nf, TypeError> {
        (**self).read_back(prg).map(Rc::new)
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

impl<'a> Print<'a> for Val {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
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

    #[trace("↓{:P} ~> {return:P}", self, std::convert::identity)]
    fn read_back(&self, prg: &ast::Module) -> Result<Self::Nf, TypeError> {
        let res = match self {
            Val::TypCtor(e) => e.read_back(prg)?.into(),
            Val::Call(e) => e.read_back(prg)?.into(),
            Val::TypeUniv(e) => e.read_back(prg)?.into(),
            Val::LocalComatch(e) => e.read_back(prg)?.into(),
            Val::Neu(exp) => exp.read_back(prg)?,
        };
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

impl<'a> Print<'a> for TypCtor {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
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

impl<'a> Print<'a> for Call {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
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

impl<'a> Print<'a> for TypeUniv {
    fn print(&'a self, _cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
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
    // TODO: Ignore this field for PartialEq, Hash?
    pub body: Match,
}

impl Shift for LocalComatch {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let LocalComatch { span, name, is_lambda_sugar, body } = self;
        LocalComatch {
            span: *span,
            name: name.clone(),
            is_lambda_sugar: *is_lambda_sugar,
            body: body.shift_in_range(range, by),
        }
    }
}

impl<'a> Print<'a> for LocalComatch {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let LocalComatch { span: _, name, is_lambda_sugar: _, body } = self;
        alloc
            .keyword(COMATCH)
            .append(alloc.space())
            .append(alloc.text(name.to_string()))
            .append(alloc.space())
            .append(body.print(cfg, alloc))
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
        let LocalComatch { span, name, is_lambda_sugar, body } = self;
        Ok(ast::LocalComatch {
            span: *span,
            ctx: None,
            name: name.clone(),
            is_lambda_sugar: *is_lambda_sugar,
            body: body.read_back(prg)?,
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
}

impl Shift for Neu {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        match self {
            Neu::Variable(e) => e.shift_in_range(range, by).into(),
            Neu::DotCall(e) => e.shift_in_range(range, by).into(),
            Neu::LocalMatch(e) => e.shift_in_range(range, by).into(),
            Neu::Hole(e) => e.shift_in_range(range, by).into(),
        }
    }
}

impl<'a> Print<'a> for Neu {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        match self {
            Neu::Variable(e) => e.print(cfg, alloc),
            Neu::DotCall(e) => e.print(cfg, alloc),
            Neu::LocalMatch(e) => e.print(cfg, alloc),
            Neu::Hole(e) => e.print(cfg, alloc),
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

impl<'a> Print<'a> for Variable {
    fn print(&'a self, _cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
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

impl<'a> Print<'a> for DotCall {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
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
    // TODO: Ignore this field for PartialEq, Hash?
    pub body: Match,
}

impl Shift for LocalMatch {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let LocalMatch { span, name, on_exp, body } = self;
        LocalMatch {
            span: *span,
            name: name.clone(),
            on_exp: on_exp.shift_in_range(range.clone(), by),
            body: body.shift_in_range(range, by),
        }
    }
}

impl<'a> Print<'a> for LocalMatch {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let LocalMatch { span: _, name, on_exp, body } = self;
        on_exp
            .print(cfg, alloc)
            .append(DOT)
            .append(alloc.keyword(MATCH))
            .append(alloc.space())
            .append(alloc.text(name.to_string()))
            .append(alloc.space())
            .append(body.print(cfg, alloc))
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
        let LocalMatch { span, name, on_exp, body } = self;
        Ok(ast::LocalMatch {
            span: *span,
            ctx: None,
            motive: None,
            ret_typ: None,
            name: name.clone(),
            on_exp: on_exp.read_back(prg)?,
            body: body.read_back(prg)?,
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
    pub metavar: Option<MetaVar>,
    /// Explicit substitution of the context, compare documentation of ast::Hole
    pub args: Vec<Vec<Rc<Val>>>,
}

impl Shift for Hole {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let Hole { span, metavar, args } = self;
        Hole { span: *span, metavar: *metavar, args: args.shift_in_range(range, by) }
    }
}

impl<'a> Print<'a> for Hole {
    fn print(&'a self, _cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        alloc.keyword(HOLE)
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
        let Hole { span, metavar, args } = self;
        let args = args.read_back(prg)?;
        Ok(ast::Hole {
            span: *span,
            metavar: *metavar,
            inferred_type: None,
            inferred_ctx: None,
            args,
        })
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

impl<'a> Print<'a> for Match {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Match { span: _, cases, omit_absurd } = self;
        let sep = alloc.text(COMMA).append(alloc.hardline());
        alloc
            .hardline()
            .append(alloc.intersperse(cases.iter().map(|x| x.print(cfg, alloc)), sep))
            .append(if *omit_absurd {
                alloc.text(COMMA).append(alloc.text("..")).append(alloc.keyword(ABSURD))
            } else {
                alloc.nil()
            })
            .nest(cfg.indent)
            .append(alloc.hardline())
            .braces_anno()
    }
}

impl ReadBack for Match {
    type Nf = ast::Match;

    fn read_back(&self, prg: &ast::Module) -> Result<Self::Nf, TypeError> {
        let Match { span, cases, omit_absurd } = self;
        Ok(ast::Match { span: *span, cases: cases.read_back(prg)?, omit_absurd: *omit_absurd })
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
    pub name: ast::Ident,
    pub params: ast::TelescopeInst,
    /// Body being `None` represents an absurd pattern
    pub body: Option<Closure>,
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

impl<'a> Print<'a> for Case {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Case { span: _, name, params, body } = self;

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
        let Case { span, name, params, body } = self;

        Ok(ast::Case {
            span: *span,
            name: name.clone(),
            params: params.clone(),
            body: body.read_back(prg)?,
        })
    }
}

// Args
//
//

pub type Args = Vec<Rc<Val>>;

// Closure
//
//

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Closure {
    pub env: Env,
    pub n_args: usize,
    pub body: Rc<ast::Exp>,
}

impl Shift for Closure {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let Closure { env, n_args, body } = self;

        Closure { env: env.shift_in_range(range, by), n_args: *n_args, body: body.clone() }
    }
}

impl<'a> Print<'a> for Closure {
    fn print(&'a self, _cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        alloc.text("...")
    }
}

impl ReadBack for Closure {
    type Nf = Rc<ast::Exp>;

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
