use std::rc::Rc;

use derivative::Derivative;

use crate::normalizer::env::*;
use codespan::Span;
use pretty::DocAllocator;
use printer::theme::ThemeExt;
use printer::tokens::*;
use printer::types::*;
use printer::util::*;
use syntax::common::*;
use syntax::ust;

/// The result of evaluation
#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub enum Val {
    TypCtor {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        span: Option<Span>,
        name: ust::Ident,
        args: Args,
    },
    Ctor {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        span: Option<Span>,
        name: ust::Ident,
        args: Args,
    },
    Type {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        span: Option<Span>,
    },
    Comatch {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        span: Option<Span>,
        name: ust::Label,
        is_lambda_sugar: bool,
        // TODO: Ignore this field for PartialEq, Hash?
        body: Match,
    },
    Neu {
        exp: Neu,
    },
}

/// A term whose evaluation is blocked
#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub enum Neu {
    Var {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        span: Option<Span>,
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        name: ust::Ident,
        idx: Idx,
    },
    Dtor {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        span: Option<Span>,
        exp: Rc<Neu>,
        name: ust::Ident,
        args: Args,
    },
    Match {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        span: Option<Span>,
        name: ust::Label,
        on_exp: Rc<Neu>,
        // TODO: Ignore this field for PartialEq, Hash?
        body: Match,
    },
    Hole {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        span: Option<Span>,
    },
}

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Match {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub span: Option<Span>,
    pub cases: Vec<Case>,
    pub omit_absurd: bool,
}

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Case {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub span: Option<Span>,
    pub name: ust::Ident,
    // TODO: Rename to params
    pub args: ust::TelescopeInst,
    /// Body being `None` represents an absurd pattern
    pub body: Option<Closure>,
}

#[derive(Debug, Clone)]
pub struct TypApp {
    pub span: Option<Span>,
    pub name: ust::Ident,
    pub args: Args,
}

pub type Args = Vec<Rc<Val>>;

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Closure {
    pub env: Env,
    pub n_args: usize,
    pub body: Rc<ust::Exp>,
}

impl Shift for Val {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        match self {
            Val::TypCtor { span, name, args } => Val::TypCtor {
                span: *span,
                name: name.clone(),
                args: args.shift_in_range(range, by),
            },
            Val::Ctor { span, name, args } => {
                Val::Ctor { span: *span, name: name.clone(), args: args.shift_in_range(range, by) }
            }
            Val::Type { span } => Val::Type { span: *span },
            Val::Comatch { span, name, is_lambda_sugar, body } => Val::Comatch {
                span: *span,
                name: name.clone(),
                is_lambda_sugar: *is_lambda_sugar,
                body: body.shift_in_range(range, by),
            },
            Val::Neu { exp } => Val::Neu { exp: exp.shift_in_range(range, by) },
        }
    }
}

impl Shift for Neu {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        match self {
            Neu::Var { span, name, idx } => {
                Neu::Var { span: *span, name: name.clone(), idx: idx.shift_in_range(range, by) }
            }
            Neu::Dtor { span, exp, name, args } => Neu::Dtor {
                span: *span,
                exp: exp.shift_in_range(range.clone(), by),
                name: name.clone(),
                args: args.shift_in_range(range, by),
            },
            Neu::Match { span, name, on_exp, body } => Neu::Match {
                span: *span,
                name: name.clone(),
                on_exp: on_exp.shift_in_range(range.clone(), by),
                body: body.shift_in_range(range, by),
            },
            Neu::Hole { span } => Neu::Hole { span: *span },
        }
    }
}

impl Shift for Match {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let Match { span, cases, omit_absurd } = self;
        Match { span: *span, cases: cases.shift_in_range(range, by), omit_absurd: *omit_absurd }
    }
}

impl Shift for Case {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let Case { span, name, args, body } = self;

        Case {
            span: *span,
            name: name.clone(),
            args: args.clone(),
            body: body.shift_in_range(range.shift(1), by),
        }
    }
}

impl Shift for Closure {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let Closure { env, n_args, body } = self;

        Closure { env: env.shift_in_range(range, by), n_args: *n_args, body: body.clone() }
    }
}
impl<'a> Print<'a> for Val {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        match self {
            Val::TypCtor { span: _, name, args } => {
                let psubst =
                    if args.is_empty() { alloc.nil() } else { args.print(cfg, alloc).parens() };
                alloc.typ(name).append(psubst)
            }
            Val::Ctor { span: _, name, args } => {
                let psubst =
                    if args.is_empty() { alloc.nil() } else { args.print(cfg, alloc).parens() };
                alloc.ctor(name).append(psubst)
            }
            Val::Type { span: _ } => alloc.typ(TYPE),
            Val::Comatch { span: _, name, is_lambda_sugar: _, body } => alloc
                .keyword(COMATCH)
                .append(alloc.space())
                .append(alloc.text(name.to_string()))
                .append(alloc.space())
                .append(body.print(cfg, alloc)),
            Val::Neu { exp } => exp.print(cfg, alloc),
        }
    }
}

impl<'a> Print<'a> for Neu {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        match self {
            Neu::Var { span: _, name, idx } => alloc.text(format!("{name}@{idx}")),
            Neu::Dtor { span: _, exp, name, args } => {
                let psubst =
                    if args.is_empty() { alloc.nil() } else { args.print(cfg, alloc).parens() };
                exp.print(cfg, alloc).append(DOT).append(alloc.dtor(name)).append(psubst)
            }
            Neu::Match { span: _, name, on_exp, body } => on_exp
                .print(cfg, alloc)
                .append(DOT)
                .append(alloc.keyword(MATCH))
                .append(alloc.space())
                .append(alloc.text(name.to_string()))
                .append(alloc.space())
                .append(body.print(cfg, alloc)),
            Neu::Hole { .. } => alloc.keyword(HOLE),
        }
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

impl<'a> Print<'a> for Case {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Case { span: _, name, args, body } = self;

        let body = match body {
            None => alloc.keyword(ABSURD),
            Some(body) => alloc
                .text(FAT_ARROW)
                .append(alloc.line())
                .append(body.print(cfg, alloc))
                .nest(cfg.indent),
        };

        alloc.ctor(name).append(args.print(cfg, alloc)).append(alloc.space()).append(body).group()
    }
}

impl<'a> Print<'a> for Closure {
    fn print(&'a self, _cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        alloc.text("...")
    }
}
