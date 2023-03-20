use std::rc::Rc;

use derivative::Derivative;

use crate::env::*;
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
        info: Info,
        name: Ident,
        args: Args,
    },
    Ctor {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        info: Info,
        name: Ident,
        args: Args,
    },
    Type {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        info: Info,
    },
    Comatch {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        info: Info,
        name: Ident,
        is_lambda_sugar: bool,
        // TODO: Ignore this field for PartialEq, Hash?
        body: Comatch,
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
        info: Info,
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        name: Ident,
        idx: Idx,
    },
    Dtor {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        info: ust::Info,
        exp: Rc<Neu>,
        name: Ident,
        args: Args,
    },
    Match {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        info: Info,
        name: Ident,
        on_exp: Rc<Neu>,
        // TODO: Ignore this field for PartialEq, Hash?
        body: Match,
    },
    Hole {
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        info: Info,
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        kind: HoleKind,
    },
}

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Match {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub info: Info,
    pub cases: Vec<Case>,
}

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Comatch {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub info: Info,
    // TODO: Consider renaming this field to "cocases"
    pub cases: Vec<Cocase>,
}

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Case {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub info: Info,
    pub name: Ident,
    // TODO: Rename to params
    pub args: ust::TelescopeInst,
    /// Body being `None` represents an absurd pattern
    pub body: Option<Closure>,
}

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Cocase {
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub info: Info,
    pub name: Ident,
    // TODO: Rename to params
    pub args: ust::TelescopeInst,
    /// Body being `None` represents an absurd pattern
    pub body: Option<Closure>,
}

#[derive(Debug, Clone)]
pub struct TypApp {
    pub info: Info,
    pub name: Ident,
    pub args: Args,
}

pub type Info = ust::Info;
pub type Args = Vec<Rc<Val>>;

#[derive(Debug, Clone, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct Closure {
    pub env: Env,
    pub n_args: usize,
    pub body: Rc<ust::Exp>,
}

impl ShiftInRange for Val {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        match self {
            Val::TypCtor { info, name, args } => Val::TypCtor {
                info: info.clone(),
                name: name.clone(),
                args: args.shift_in_range(range, by),
            },
            Val::Ctor { info, name, args } => Val::Ctor {
                info: info.clone(),
                name: name.clone(),
                args: args.shift_in_range(range, by),
            },
            Val::Type { info } => Val::Type { info: info.clone() },
            Val::Comatch { info, name, is_lambda_sugar, body } => Val::Comatch {
                info: info.clone(),
                name: name.clone(),
                is_lambda_sugar: *is_lambda_sugar,
                body: body.shift_in_range(range, by),
            },
            Val::Neu { exp } => Val::Neu { exp: exp.shift_in_range(range, by) },
        }
    }
}

impl ShiftInRange for Neu {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        match self {
            Neu::Var { info, name, idx } => Neu::Var {
                info: info.clone(),
                name: name.clone(),
                idx: idx.shift_in_range(range, by),
            },
            Neu::Dtor { info, exp, name, args } => Neu::Dtor {
                info: info.clone(),
                exp: exp.shift_in_range(range.clone(), by),
                name: name.clone(),
                args: args.shift_in_range(range, by),
            },
            Neu::Match { info, name, on_exp, body } => Neu::Match {
                info: info.clone(),
                name: name.clone(),
                on_exp: on_exp.shift_in_range(range.clone(), by),
                body: body.shift_in_range(range, by),
            },
            Neu::Hole { info, kind } => Neu::Hole { info: info.clone(), kind: *kind },
        }
    }
}

impl ShiftInRange for Match {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let Match { info, cases } = self;
        Match { info: info.clone(), cases: cases.shift_in_range(range, by) }
    }
}

impl ShiftInRange for Comatch {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let Comatch { info, cases } = self;
        Comatch { info: info.clone(), cases: cases.shift_in_range(range, by) }
    }
}

impl ShiftInRange for Case {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let Case { info, name, args, body } = self;

        Case {
            info: info.clone(),
            name: name.clone(),
            args: args.clone(),
            body: body.shift_in_range(range.shift(1), by),
        }
    }
}

impl ShiftInRange for Cocase {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let Cocase { info, name, args, body } = self;

        Cocase {
            info: info.clone(),
            name: name.clone(),
            args: args.clone(),
            body: body.shift_in_range(range.shift(1), by),
        }
    }
}

impl ShiftInRange for Closure {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let Closure { env, n_args, body } = self;

        Closure { env: env.shift_in_range(range, by), n_args: *n_args, body: body.clone() }
    }
}
impl<'a> Print<'a> for Val {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        match self {
            Val::TypCtor { info: _, name, args } => {
                let psubst =
                    if args.is_empty() { alloc.nil() } else { args.print(cfg, alloc).parens() };
                alloc.typ(name).append(psubst)
            }
            Val::Ctor { info: _, name, args } => {
                let psubst =
                    if args.is_empty() { alloc.nil() } else { args.print(cfg, alloc).parens() };
                alloc.ctor(name).append(psubst)
            }
            Val::Type { info: _ } => alloc.typ(TYPE),
            Val::Comatch { info: _, name, is_lambda_sugar: _, body } => alloc
                .keyword(COMATCH)
                .append(alloc.space())
                .append(alloc.text(name))
                .append(alloc.space())
                .append(body.print(cfg, alloc)),
            Val::Neu { exp } => exp.print(cfg, alloc),
        }
    }
}

impl<'a> Print<'a> for Neu {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        match self {
            Neu::Var { info: _, name, idx } => alloc.text(format!("{name}@{idx}")),
            Neu::Dtor { info: _, exp, name, args } => {
                let psubst =
                    if args.is_empty() { alloc.nil() } else { args.print(cfg, alloc).parens() };
                exp.print(cfg, alloc).append(DOT).append(alloc.dtor(name)).append(psubst)
            }
            Neu::Match { info: _, name, on_exp, body } => on_exp
                .print(cfg, alloc)
                .append(DOT)
                .append(alloc.keyword(MATCH))
                .append(alloc.space())
                .append(alloc.text(name))
                .append(alloc.space())
                .append(body.print(cfg, alloc)),
            Neu::Hole { info: _, kind } => match kind {
                syntax::common::HoleKind::Todo => alloc.keyword(HOLE_TODO),
                syntax::common::HoleKind::Omitted => alloc.keyword(HOLE_OMITTED),
            },
        }
    }
}
impl<'a> Print<'a> for Comatch {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Comatch { info: _, cases } = self;
        let sep = alloc.text(COMMA).append(alloc.hardline());

        alloc
            .hardline()
            .append(alloc.intersperse(cases.iter().map(|x| x.print(cfg, alloc)), sep))
            .nest(cfg.indent)
            .append(alloc.hardline())
            .braces_from(cfg)
    }
}

impl<'a> Print<'a> for Match {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Match { info: _, cases } = self;
        let sep = alloc.text(COMMA).append(alloc.hardline());
        alloc
            .hardline()
            .append(alloc.intersperse(cases.iter().map(|x| x.print(cfg, alloc)), sep))
            .nest(cfg.indent)
            .append(alloc.hardline())
            .braces_from(cfg)
    }
}

impl<'a> Print<'a> for Case {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Case { info: _, name, args, body } = self;

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

impl<'a> Print<'a> for Cocase {
    fn print(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let Cocase { info: _, name, args, body } = self;

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
