use std::rc::Rc;

use ast;
use ast::ctx::BindContext;
use ast::shift_and_clone;
use ast::Idx;
use ast::MetaVar;
use ast::Shift;
use ast::ShiftRange;
use ast::ShiftRangeExt;
use ast::VarBound;
use codespan::Span;
use log::trace;
use pretty::DocAllocator;
use printer::theme::ThemeExt;
use printer::tokens::*;
use printer::types::Print;
use printer::types::*;
use printer::util::*;

use crate::normalizer::env::*;
use crate::TypeInfoTable;

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

    fn read_back(&self, info_table: &Rc<TypeInfoTable>) -> Result<Self::Nf, TypeError>;
}

impl<T: ReadBack> ReadBack for Vec<T> {
    type Nf = Vec<T::Nf>;

    fn read_back(&self, info_table: &Rc<TypeInfoTable>) -> Result<Self::Nf, TypeError> {
        self.iter().map(|x| x.read_back(info_table)).collect()
    }
}

impl<T: ReadBack> ReadBack for Box<T> {
    type Nf = Box<T::Nf>;

    fn read_back(&self, info_table: &Rc<TypeInfoTable>) -> Result<Self::Nf, TypeError> {
        (**self).read_back(info_table).map(Box::new)
    }
}

impl<T: ReadBack> ReadBack for Option<T> {
    type Nf = Option<T::Nf>;

    fn read_back(&self, info_table: &Rc<TypeInfoTable>) -> Result<Self::Nf, TypeError> {
        self.as_ref().map(|x| x.read_back(info_table)).transpose()
    }
}

// Val
//
//

/// The result of evaluation
#[derive(Debug, Clone)]
pub enum Val {
    TypCtor(TypCtor),
    // A call is only a value if it is a constructor or a codefinition.
    Call(Call),
    TypeUniv(TypeUniv),
    LocalComatch(LocalComatch),
    Anno(Anno),
    Neu(Neu),
}

impl Shift for Val {
    fn shift_in_range<R: ShiftRange>(&mut self, range: &R, by: (isize, isize)) {
        match self {
            Val::TypCtor(e) => e.shift_in_range(range, by),
            Val::Call(e) => e.shift_in_range(range, by),
            Val::TypeUniv(e) => e.shift_in_range(range, by),
            Val::LocalComatch(e) => e.shift_in_range(range, by),
            Val::Anno(e) => e.shift_in_range(range, by),
            Val::Neu(exp) => exp.shift_in_range(range, by),
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
            Val::Anno(e) => e.print(cfg, alloc),
            Val::Neu(exp) => exp.print(cfg, alloc),
        }
    }
}

impl ReadBack for Val {
    type Nf = ast::Exp;

    fn read_back(&self, info_table: &Rc<TypeInfoTable>) -> Result<Self::Nf, TypeError> {
        let res = match self {
            Val::TypCtor(e) => e.read_back(info_table)?.into(),
            Val::Call(e) => e.read_back(info_table)?.into(),
            Val::TypeUniv(e) => e.read_back(info_table)?.into(),
            Val::LocalComatch(e) => e.read_back(info_table)?.into(),
            Val::Anno(e) => e.read_back(info_table)?.into(),
            Val::Neu(exp) => exp.read_back(info_table)?,
        };
        trace!("↓{} ~> {}", self.print_trace(), res.print_trace());
        Ok(res)
    }
}

// TypCtor
//
//

#[derive(Debug, Clone)]
pub struct TypCtor {
    pub span: Option<Span>,
    pub name: ast::IdBound,
    pub args: Args,
}

impl Shift for TypCtor {
    fn shift_in_range<R: ShiftRange>(&mut self, range: &R, by: (isize, isize)) {
        self.args.shift_in_range(range, by);
    }
}

impl Print for TypCtor {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let TypCtor { span: _, name, args } = self;
        let psubst = if args.is_empty() { alloc.nil() } else { args.print(cfg, alloc).parens() };
        alloc.typ(&name.id).append(psubst)
    }
}

impl From<TypCtor> for Val {
    fn from(value: TypCtor) -> Self {
        Val::TypCtor(value)
    }
}

impl ReadBack for TypCtor {
    type Nf = ast::TypCtor;

    fn read_back(&self, info_table: &Rc<TypeInfoTable>) -> Result<Self::Nf, TypeError> {
        let TypCtor { span, name, args } = self;
        Ok(ast::TypCtor {
            span: *span,
            name: name.clone(),
            args: ast::Args { args: args.read_back(info_table)? },
        })
    }
}

// Call
//
//

#[derive(Debug, Clone)]
pub struct Call {
    pub span: Option<Span>,
    pub kind: ast::CallKind,
    pub name: ast::IdBound,
    pub args: Args,
}

impl Shift for Call {
    fn shift_in_range<R: ShiftRange>(&mut self, range: &R, by: (isize, isize)) {
        self.args.shift_in_range(range, by);
    }
}

impl Print for Call {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Call { span: _, kind: _, name, args } = self;
        let psubst = if args.is_empty() { alloc.nil() } else { args.print(cfg, alloc).parens() };
        alloc.ctor(&name.id).append(psubst)
    }
}

impl From<Call> for Val {
    fn from(value: Call) -> Self {
        Val::Call(value)
    }
}

impl ReadBack for Call {
    type Nf = ast::Call;

    fn read_back(&self, info_table: &Rc<TypeInfoTable>) -> Result<Self::Nf, TypeError> {
        let Call { span, kind, name, args } = self;
        Ok(ast::Call {
            span: *span,
            kind: *kind,
            name: name.clone(),
            args: ast::Args { args: args.read_back(info_table)? },
            inferred_type: None,
        })
    }
}

// TypeUniv
//
//

#[derive(Debug, Clone)]
pub struct TypeUniv {
    pub span: Option<Span>,
}

impl Shift for TypeUniv {
    fn shift_in_range<R: ShiftRange>(&mut self, _range: &R, _by: (isize, isize)) {}
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

    fn read_back(&self, _info_table: &Rc<TypeInfoTable>) -> Result<Self::Nf, TypeError> {
        let TypeUniv { span } = self;
        Ok(ast::TypeUniv { span: *span })
    }
}

// LocalComatch
//
//

#[derive(Debug, Clone)]
pub struct LocalComatch {
    pub span: Option<Span>,
    pub name: ast::Label,
    pub is_lambda_sugar: bool,
    pub cases: Vec<Case>,
}

impl Shift for LocalComatch {
    fn shift_in_range<R: ShiftRange>(&mut self, range: &R, by: (isize, isize)) {
        self.cases.shift_in_range(range, by);
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
    fn read_back(&self, info_table: &Rc<TypeInfoTable>) -> Result<Self::Nf, TypeError> {
        let LocalComatch { span, name, is_lambda_sugar, cases } = self;
        Ok(ast::LocalComatch {
            span: *span,
            ctx: None,
            name: name.clone(),
            is_lambda_sugar: *is_lambda_sugar,
            cases: cases.read_back(info_table)?,
            inferred_type: None,
        })
    }
}

// Anno
//
//

#[derive(Debug, Clone)]
pub struct Anno {
    pub span: Option<Span>,
    pub exp: Box<Val>,
    pub typ: Box<Val>,
}

impl Shift for Anno {
    fn shift_in_range<R: ShiftRange>(&mut self, range: &R, by: (isize, isize)) {
        self.exp.shift_in_range(range, by);
        self.typ.shift_in_range(range, by);
    }
}

impl Print for Anno {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Anno { span: _, exp, typ } = self;
        exp.print(cfg, alloc).parens().append(COLON).append(typ.print(cfg, alloc))
    }
}

impl From<Anno> for Val {
    fn from(value: Anno) -> Self {
        Val::Anno(value)
    }
}

impl ReadBack for Anno {
    type Nf = ast::Anno;

    fn read_back(&self, info_table: &Rc<TypeInfoTable>) -> Result<Self::Nf, TypeError> {
        let Anno { span, exp, typ } = self;
        let typ_nf = typ.read_back(info_table)?;
        Ok(ast::Anno {
            span: *span,
            exp: exp.read_back(info_table)?,
            typ: typ_nf.clone(),
            normalized_type: Some(typ_nf),
        })
    }
}

// Neu
//
//

/// A term whose evaluation is blocked
#[derive(Debug, Clone)]
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
    fn shift_in_range<R: ShiftRange>(&mut self, range: &R, by: (isize, isize)) {
        match self {
            Neu::Variable(e) => e.shift_in_range(range, by),
            Neu::DotCall(e) => e.shift_in_range(range, by),
            Neu::LocalMatch(e) => e.shift_in_range(range, by),
            Neu::Hole(e) => e.shift_in_range(range, by),
            Neu::OpaqueCall(e) => e.shift_in_range(range, by),
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

    fn read_back(&self, info_table: &Rc<TypeInfoTable>) -> Result<Self::Nf, TypeError> {
        let res = match self {
            Neu::Variable(e) => e.read_back(info_table)?.into(),
            Neu::DotCall(e) => e.read_back(info_table)?.into(),
            Neu::LocalMatch(e) => e.read_back(info_table)?.into(),
            Neu::Hole(e) => e.read_back(info_table)?.into(),
            Neu::OpaqueCall(e) => e.read_back(info_table)?.into(),
        };
        Ok(res)
    }
}

// Variable
//
//

#[derive(Debug, Clone)]
pub struct Variable {
    pub span: Option<Span>,
    pub name: ast::VarBound,
    pub idx: Idx,
}

impl Shift for Variable {
    fn shift_in_range<R: ShiftRange>(&mut self, range: &R, by: (isize, isize)) {
        self.idx.shift_in_range(range, by);
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

    fn read_back(&self, _info_table: &Rc<TypeInfoTable>) -> Result<Self::Nf, TypeError> {
        let Variable { span, name, idx } = self;
        Ok(ast::Variable { span: *span, idx: *idx, name: name.clone(), inferred_type: None })
    }
}

// DotCall
//
//

#[derive(Debug, Clone)]
pub struct DotCall {
    pub span: Option<Span>,
    pub kind: ast::DotCallKind,
    pub exp: Box<Neu>,
    pub name: ast::IdBound,
    pub args: Args,
}

impl Shift for DotCall {
    fn shift_in_range<R: ShiftRange>(&mut self, range: &R, by: (isize, isize)) {
        self.exp.shift_in_range(range, by);
        self.args.shift_in_range(range, by);
    }
}

impl Print for DotCall {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let DotCall { span: _, kind: _, exp, name, args } = self;
        let psubst = if args.is_empty() { alloc.nil() } else { args.print(cfg, alloc).parens() };
        exp.print(cfg, alloc).append(DOT).append(alloc.dtor(&name.id)).append(psubst)
    }
}

impl From<DotCall> for Neu {
    fn from(value: DotCall) -> Self {
        Neu::DotCall(value)
    }
}

impl ReadBack for DotCall {
    type Nf = ast::DotCall;

    fn read_back(&self, info_table: &Rc<TypeInfoTable>) -> Result<Self::Nf, TypeError> {
        let DotCall { span, kind, exp, name, args } = self;
        Ok(ast::DotCall {
            span: *span,
            kind: *kind,
            exp: exp.read_back(info_table)?,
            name: name.clone(),
            args: ast::Args { args: args.read_back(info_table)? },
            inferred_type: None,
        })
    }
}

// LocalMatch
//
//

#[derive(Debug, Clone)]
pub struct LocalMatch {
    pub span: Option<Span>,
    pub name: ast::Label,
    pub on_exp: Box<Neu>,
    pub cases: Vec<Case>,
}

impl Shift for LocalMatch {
    fn shift_in_range<R: ShiftRange>(&mut self, range: &R, by: (isize, isize)) {
        self.on_exp.shift_in_range(range, by);
        self.cases.shift_in_range(range, by);
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

    fn read_back(&self, info_table: &Rc<TypeInfoTable>) -> Result<Self::Nf, TypeError> {
        let LocalMatch { span, name, on_exp, cases } = self;
        Ok(ast::LocalMatch {
            span: *span,
            ctx: None,
            motive: None,
            ret_typ: None,
            name: name.clone(),
            on_exp: on_exp.read_back(info_table)?,
            cases: cases.read_back(info_table)?,
            inferred_type: None,
        })
    }
}

// Hole
//
//

#[derive(Debug, Clone)]
pub struct Hole {
    pub span: Option<Span>,
    pub kind: ast::MetaVarKind,
    pub metavar: MetaVar,
    /// Explicit substitution of the context, compare documentation of ast::Hole
    pub args: Vec<Vec<Box<Val>>>,
}

impl Shift for Hole {
    fn shift_in_range<R: ShiftRange>(&mut self, range: &R, by: (isize, isize)) {
        self.args.shift_in_range(range, by);
    }
}

impl Print for Hole {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        if cfg.print_metavar_ids {
            alloc.text(format!("?{}", self.metavar.id))
        } else {
            alloc.keyword(QUESTION_MARK)
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

    fn read_back(&self, info_table: &Rc<TypeInfoTable>) -> Result<Self::Nf, TypeError> {
        let Hole { span, kind, metavar, args } = self;
        let args = args.read_back(info_table)?;
        Ok(ast::Hole {
            span: *span,
            kind: *kind,
            metavar: *metavar,
            inferred_type: None,
            inferred_ctx: None,
            args,
            solution: None,
        })
    }
}

// Case
//
//

#[derive(Debug, Clone)]
pub struct Case {
    pub span: Option<Span>,
    pub is_copattern: bool,
    pub name: ast::IdBound,
    pub params: ast::TelescopeInst,
    /// Body being `None` represents an absurd pattern
    pub body: Option<Closure>,
}

impl Shift for Case {
    fn shift_in_range<R: ShiftRange>(&mut self, range: &R, by: (isize, isize)) {
        let range = (*range).clone();
        self.body.shift_in_range(&range.shift(1), by);
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

        alloc
            .ctor(&name.id)
            .append(params.print(cfg, alloc))
            .append(alloc.space())
            .append(body)
            .group()
    }
}

impl ReadBack for Case {
    type Nf = ast::Case;

    fn read_back(&self, info_table: &Rc<TypeInfoTable>) -> Result<Self::Nf, TypeError> {
        let Case { span, is_copattern, name, params, body } = self;

        Ok(ast::Case {
            span: *span,
            pattern: ast::Pattern {
                is_copattern: *is_copattern,
                name: name.clone(),
                params: params.clone(),
            },
            body: body.read_back(info_table)?,
        })
    }
}

// OpaqueCall
//
//

#[derive(Debug, Clone)]
pub struct OpaqueCall {
    pub span: Option<Span>,
    pub name: ast::IdBound,
    pub args: Args,
}

impl Shift for OpaqueCall {
    fn shift_in_range<R: ShiftRange>(&mut self, range: &R, by: (isize, isize)) {
        self.args.shift_in_range(range, by);
    }
}

impl Print for OpaqueCall {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let OpaqueCall { span: _, name, args } = self;
        let psubst = if args.is_empty() { alloc.nil() } else { args.print(cfg, alloc).parens() };
        alloc.ctor(&name.id).append(psubst)
    }
}

impl From<OpaqueCall> for Neu {
    fn from(value: OpaqueCall) -> Self {
        Neu::OpaqueCall(value)
    }
}

impl ReadBack for OpaqueCall {
    type Nf = ast::Call;

    fn read_back(&self, info_table: &Rc<TypeInfoTable>) -> Result<Self::Nf, TypeError> {
        let OpaqueCall { span, name, args } = self;
        Ok(ast::Call {
            span: *span,
            kind: ast::CallKind::LetBound,
            name: name.clone(),
            args: ast::Args { args: args.read_back(info_table)? },
            inferred_type: None,
        })
    }
}

// Args
//
//

#[derive(Debug, Clone)]
pub struct Args(pub Vec<Arg>);

impl Args {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn to_vals(&self) -> Vec<Box<Val>> {
        self.0.iter().map(Arg::to_val).collect()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Arg> {
        self.0.iter()
    }
}

impl Shift for Args {
    fn shift_in_range<R: ShiftRange>(&mut self, range: &R, by: (isize, isize)) {
        self.0.shift_in_range(range, by)
    }
}

impl ReadBack for Args {
    type Nf = Vec<ast::Arg>;

    fn read_back(&self, info_table: &Rc<TypeInfoTable>) -> Result<Self::Nf, TypeError> {
        self.0.read_back(info_table)
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

#[derive(Debug, Clone)]
pub enum Arg {
    UnnamedArg(Box<Val>),
    NamedArg(ast::VarBound, Box<Val>),
    InsertedImplicitArg(Box<Val>),
}

impl Arg {
    pub fn is_inserted_implicit(&self) -> bool {
        matches!(self, Arg::InsertedImplicitArg(_))
    }
}

impl Shift for Arg {
    fn shift_in_range<R: ShiftRange>(&mut self, range: &R, by: (isize, isize)) {
        match self {
            Arg::UnnamedArg(val) => val.shift_in_range(range, by),
            Arg::NamedArg(_, val) => val.shift_in_range(range, by),
            Arg::InsertedImplicitArg(val) => val.shift_in_range(range, by),
        }
    }
}

impl ReadBack for Arg {
    type Nf = ast::Arg;

    fn read_back(&self, info_table: &Rc<TypeInfoTable>) -> Result<Self::Nf, TypeError> {
        match self {
            Arg::UnnamedArg(val) => Ok(ast::Arg::UnnamedArg(val.read_back(info_table)?)),
            Arg::NamedArg(name, val) => {
                Ok(ast::Arg::NamedArg(name.clone(), val.read_back(info_table)?))
            }
            Arg::InsertedImplicitArg(val) => Ok(ast::Arg::UnnamedArg(val.read_back(info_table)?)),
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
    pub fn to_val(&self) -> Box<Val> {
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

#[derive(Debug, Clone)]
pub struct Closure {
    pub env: Env,
    pub n_args: usize,
    pub body: Box<ast::Exp>,
}

impl Shift for Closure {
    fn shift_in_range<R: ShiftRange>(&mut self, range: &R, by: (isize, isize)) {
        self.env.shift_in_range(range, by);
    }
}

impl Print for Closure {
    fn print<'a>(&'a self, _cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        alloc.text("...")
    }
}

impl ReadBack for Closure {
    type Nf = Box<ast::Exp>;

    fn read_back(&self, info_table: &Rc<TypeInfoTable>) -> Result<Self::Nf, TypeError> {
        let args: Vec<Box<Val>> = (0..self.n_args)
            .rev()
            .map(|snd| {
                Val::Neu(Neu::Variable(Variable {
                    span: None,
                    name: VarBound::from_string(""),
                    idx: Idx { fst: 0, snd },
                }))
            })
            .map(Box::new)
            .collect();
        let mut shifted_env = shift_and_clone(&self.env, (1, 0));
        shifted_env
            .bind_iter(args.iter(), |env| self.body.eval(info_table, env))?
            .read_back(info_table)
    }
}
